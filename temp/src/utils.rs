#![allow(unused)]

use anyhow::Result;
use bincode::{
    Decode, Encode,
    config::standard,
    serde::{decode_from_slice, encode_to_vec},
};
use hound::WavReader;
use kdtree::KdTree;
use kdtree::distance::squared_euclidean;
use realfft::{RealFftPlanner, num_complex::Complex};
use serde::{Deserialize, Serialize};
use sled::IVec;
use std::{
    collections::{HashMap, hash_map::DefaultHasher},
    fs::File,
    hash::{Hash, Hasher},
    io::Cursor,
    path::Path,
    time::Instant,
};
use symphonia::{
    core::{
        audio::{AudioBuffer, AudioBufferRef, Signal},
        codecs::{CODEC_TYPE_NULL, DecoderOptions},
        conv::IntoSample,
        errors::Error as SError,
        formats::FormatOptions,
        io::MediaSourceStream,
        meta::MetadataOptions,
        probe::Hint,
    },
    default::{get_codecs, get_probe},
};

pub const BANDS: [(usize, usize); 10] = [
    (0, 32),
    (32, 64),
    (64, 128),
    (128, 256),
    (256, 512),
    (512, 1024),
    (1024, 2048),
    (2048, 4096),
    (4096, 8192),
    (8192, 20000),
];

// [
//     (0, 60),
//     (60, 250),
//     (250, 500),
//     (500, 2000),
//     (2000, 4000),
//     (4000, 6000),
//     (6000, 20000),
// ]

pub const NUM_BINS: usize = 2048;
pub const DEFAULT_SAMPLE_RATE: u32 = 44100;
pub const ANCHOR_POINTS: usize = 5;

pub(crate) fn open_file(path: &str) -> Result<MediaSourceStream> {
    let file = Box::new(File::open(path)?);
    Ok(MediaSourceStream::new(file, Default::default()))
}

pub(crate) fn open_binary(buf: Vec<u8>) -> Result<MediaSourceStream> {
    let file = Box::new(Cursor::new(buf));
    Ok(MediaSourceStream::new(file, Default::default()))
}

pub(crate) fn extract_mono_audio(mss: MediaSourceStream) -> Result<(Vec<f32>, u32)> {
    let probe = get_probe().format(&Hint::new(), mss, &Default::default(), &Default::default())?;
    let mut format = probe.format;

    let track = format
        .default_track()
        .ok_or(anyhow::anyhow!("No audio track found"))?;
    println!("Track: {:#?}", track);
    let track_id = track.id;

    let mut decoder = get_codecs()
        .make(&track.codec_params, &Default::default())
        .map_err(|e| anyhow::anyhow!("Failed to create decoder: {e}"))?;
    let sample_rate = track
        .codec_params
        .sample_rate
        .unwrap_or(DEFAULT_SAMPLE_RATE);

    let mut all_samples: Vec<f32> = vec![];

    while let Ok(packet) = format.next_packet() {
        if packet.track_id() != track_id {
            continue;
        }
        match decoder.decode(&packet) {
            Ok(decoded) => {
                match decoded {
                    AudioBufferRef::S16(buf) => {
                        let buf_i16 = buf.into_owned();
                        for frame in 0..buf_i16.frames() {
                            // Downmix to mono
                            let mut sample: i32 = 0;
                            for chan in 0..buf_i16.spec().channels.count() {
                                sample += buf_i16.chan(chan)[frame] as i32;
                            }
                            sample /= buf_i16.spec().channels.count() as i32;
                            all_samples.push(sample as f32 / i16::MAX as f32);
                        }
                    }
                    AudioBufferRef::F32(buf) => {
                        let buf_f32 = buf.into_owned();
                        for frame in 0..buf_f32.frames() {
                            // Downmix to mono
                            let mut sample: f64 = 0.0;
                            for chan in 0..buf_f32.spec().channels.count() {
                                sample += buf_f32.chan(chan)[frame] as f64;
                            }
                            sample /= buf_f32.spec().channels.count() as f64;
                            all_samples.push(sample as f32);
                        }
                    }
                    _ => return Err(anyhow::anyhow!("Unsupported audio format")),
                }
            }
            Err(e) => return Err(e.into()),
        }
    }

    Ok((all_samples, sample_rate))
}

pub(crate) fn downsample(mut audio: Vec<f32>, sr: u32, d: usize) -> (Vec<f32>, u32) {
    (
        audio
            .chunks(d)
            .map(|chunk| chunk.iter().sum::<f32>() / d as f32)
            .collect(),
        sr / d as u32,
    )
}

pub(crate) fn normalise(audio: Vec<f32>) -> Vec<f32> {
    let max = audio.iter().fold(f32::MIN, |max, &x| f32::max(max, x));
    audio.iter().map(|&x| x / max).collect()
}

pub fn hann_function(n: usize, samples: usize) -> f32 {
    0.5 * (1.0 - f32::cos((2.0 * std::f32::consts::PI * n as f32) / (samples as f32 - 1.0)))
}

pub(crate) fn stft(mut audio: Vec<f32>, fft_size: usize, hop_size: usize) -> Result<Vec<Vec<f32>>> {
    let n = audio.len();

    // compute FFT
    let mut planner = RealFftPlanner::<f32>::new();
    let r2c = planner.plan_fft_forward(fft_size);

    let window: Vec<f32> = (0..fft_size).map(|n| hann_function(n, fft_size)).collect();

    // STFT loop
    let mut stft_result = Vec::new();
    let mut frame = 0;
    while frame + fft_size <= n {
        // Windowed frame
        audio[frame..frame + fft_size]
            .iter_mut()
            .zip(&window)
            .for_each(|(x, &w)| *x *= w);
        // FFT
        let mut buf = r2c.make_output_vec();

        r2c.process(&mut audio[frame..frame + fft_size], &mut buf)
            .unwrap();
        let buf = buf.iter().map(|c| c.norm()).collect();
        stft_result.push(buf);
        frame += hop_size;
    }

    Ok(stft_result)
}

pub(crate) fn to_db(spec: Vec<Vec<f32>>) -> Vec<Vec<f32>> {
    spec.into_iter()
        .map(|frame| {
            frame
                .into_iter()
                .map(|x| 20.0 * (x.max(1e-10)).log10())
                .collect()
        })
        .collect()
}

pub(crate) fn save_spectrogram_image(name: &str, db_spec: &Vec<Vec<f32>>) -> Result<()> {
    fn colormap_jet(val: u8) -> [u8; 3] {
        let v = val as f32 / 255.0;
        let r = (4.0 * (v - 0.75)).clamp(0.0, 1.0);
        let g = (4.0 * (v - 0.5)).clamp(0.0, 1.0) - (4.0 * (v - 0.75)).clamp(0.0, 1.0);
        let b = (4.0 * (0.25 - v)).clamp(0.0, 1.0);
        [(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8]
    }

    let min_value = db_spec.iter().flatten().cloned().fold(f32::MAX, f32::min);
    let max_value = db_spec.iter().flatten().cloned().fold(f32::MIN, f32::max);

    let mut db_spec = db_spec
        .iter()
        .map(|i| {
            i.iter()
                .rev()
                .map(|&x| ((x - min_value) / (max_value - min_value) * 255.0) as u8)
                .collect::<Vec<u8>>()
        })
        .collect::<Vec<Vec<u8>>>();

    let mut buf: Vec<u8> = vec![];
    for i in 0..db_spec[0].len() {
        for j in 0..db_spec.len() {
            let rgb = colormap_jet(db_spec[j][i]);
            buf.extend_from_slice(&rgb);
        }
    }

    image::save_buffer(
        name,
        &buf,
        db_spec.len() as u32,
        db_spec[0].len() as u32,
        image::ColorType::Rgb8,
    );
    Ok(())
}

// fn get_spectogram(audio: (Vec<f32>, u32)) -> Result<Spectrogram> {
//     let mut spec = SpecOptionsBuilder::new(NUM_BINS);
//     spec = match audio {
//         Audio::I16(v, sr) => spec.load_data_from_memory(v, sr),
//         Audio::F32(v, sr) => spec.load_data_from_memory_f32(v, sr),
//     };
//     let mut spec = spec
//         .downsample(4)
//         .normalise()
//         .set_window_fn(hann_function)
//         .build()
//         .map_err(|e| anyhow::anyhow!("{:?}", e))?;
//     Ok(spec.compute())
// }

pub(crate) fn filter_stft(
    stft_result: Vec<Vec<f32>>,
    sr: usize,
    fft_size: usize,
    hop_size: usize,
) -> Vec<(usize, usize, f32)> {
    let filtered_stft = stft_result
        .iter()
        .enumerate()
        .map(|(i, frame)| {
            BANDS
                .iter()
                .map(|(min, max)| {
                    frame
                        .iter()
                        .enumerate()
                        .filter(|(j, _)| {
                            let freq = j * sr as usize / fft_size;
                            freq >= *min && freq < *max
                        })
                        .fold((0usize, 0usize, 0.0f32), |(time_, freq_, acc), (j, &x)| {
                            let time = ((i as f32 * hop_size as f32) / sr as f32 * 1000.0).floor()
                                as usize;
                            let freq = j * sr as usize / fft_size;
                            if acc < x {
                                (time, freq, x)
                            } else {
                                (time_, freq_, acc)
                            }
                        })
                })
                .filter(|&(_, _, x)| x > 1.0)
                .collect::<Vec<(usize, usize, f32)>>()
        })
        .filter(|v| !v.is_empty())
        .collect::<Vec<Vec<(usize, usize, f32)>>>();

    // filter out basic noise
    filtered_stft
        .into_iter()
        .map(|frame| {
            let avg = frame.iter().map(|&(_, _, x)| x).sum::<f32>() / frame.len() as f32;
            frame
                .into_iter()
                .filter(|&(_, _, x)| x > avg)
                .collect::<Vec<(usize, usize, f32)>>()
        })
        .filter(|v| !v.is_empty())
        .flatten()
        .collect()
}

pub fn hash_tuple(freq1: usize, freq2: usize, time: u64) -> u64 {
    let mut hash = (freq1 as u64) << 48;
    hash += ((freq2 as u64) & 0xFFFF) << 16;
    hash |= time & 0xFFFFFFFF;
    hash
}

pub(crate) fn filter_to_data(
    data: Vec<(usize, usize, f32)>,
    song_id: &str,
) -> Result<HashMap<u64, (u64, &str)>> {
    // Suppose your points are Vec<[f32; 2]> for 2D
    let mut tree = KdTree::new(2);
    for (i, (time, freq, _)) in data.iter().enumerate() {
        tree.add([*time as f32, *freq as f32], i)?;
    }

    let mut points = HashMap::new();
    for (time, freq, x) in data.iter() {
        // Find 5 because the point itself will be included as the nearest
        let nearest = tree.nearest(&[*time as f32, *freq as f32], ANCHOR_POINTS, &squared_euclidean)?;
        // Filter out the point itself if needed
        nearest.into_iter().skip(1).for_each(|(_, i)| {
            points.insert(
                hash_tuple(
                    *freq,
                    data[*i].1,
                    (*time as i64 - data[*i].0 as i64).abs() as u64,
                ),
                (*time as u64, song_id),
            );
        });
    }

    Ok(points)
}

// function to save Vec<f32> to wav
pub(crate) fn save_wav(data: Vec<f32>, sample_rate: u32, name: &str) -> Result<()> {
    let mut spec = hound::WavWriter::create(
        name,
        hound::WavSpec {
            channels: 1,
            sample_rate,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        },
    )?;
    for sample in data {
        spec.write_sample(sample)?;
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Encode, Decode)]
pub(crate) struct Data {
    time: u64,
    song_id: String,
}

//  9597952
// fn main() -> Result<()> {
//     let db = sled::open("test.db").unwrap();

//     let start = Instant::now();
//     let (samples, sr) = extract_mono_audio("All Time Low.mp3")?;
//     println!(
//         "Time taken for extract_mono_audio: {} s",
//         start.elapsed().as_secs()
//     );

//     let (samples, sr) = downsample(samples, sr, 2);
//     println!("Time taken for downsample: {} s", start.elapsed().as_secs());

//     let samples = normalise(samples);
//     println!("Time taken for normalise: {} s", start.elapsed().as_secs());

//     let stft_result = stft(samples.clone(), NUM_BINS, NUM_BINS / 2)?;
//     println!("Time taken for stft: {} s", start.elapsed().as_secs());

//     // let stft_result = to_db(stft_result);

//     // filter stft_result based on the bands
//     let filtered_stft = filter_stft(stft_result, sr as usize, NUM_BINS, NUM_BINS / 2);
//     println!(
//         "Time taken for filter_stft: {} s",
//         start.elapsed().as_secs()
//     );

//     // println!("Filtered STFT: {:?}", &filtered_stft[..50]);
//     let points = filter_to_data(filtered_stft, "All Time Low")?;
//     println!(
//         "Time taken for filter_to_data: {} s",
//         start.elapsed().as_secs()
//     );

//     for (id, (time, song_id)) in points.into_iter() {
//         if let Ok(Some(value)) = db.get(id.to_string()) {
//             let (mut value, _) : (Vec<Data>, _) = decode_from_slice(&value, standard())?;
//             value.push(Data { time, song_id: song_id.to_string() });
//             db.insert(id.to_string(), encode_to_vec(value, standard())?);
//         } else {
//             let value = vec![Data { time, song_id: song_id.to_string() }];
//             db.insert(id.to_string(), encode_to_vec(value, standard())?);
//         }
//     }

//     // println!("Number of points: {}", points.len());
//     // let (mut point, mut anchor) = (0, HashMap::new());
//     // for (id, (time, song_id)) in points.into_iter() {
//     //     if let Ok(Some(value)) = db.get(id.to_string()) {
//     //         let (mut value, _) : (Vec<Data>, _) = decode_from_slice(&value, standard())?;
//     //         *anchor.entry(value[0].time).or_insert(0) += 1;
//     //         point += 1;
//     //     }
//     // }
//     // let (mut min, mut max) = (u64::MAX, 0);
//     // for (k, v) in anchor.iter() {
//     //     if *v == 4 {
//     //         min = min.min(*k);
//     //         max = max.max(*k);
//     //     }
//     // }

//     println!("Number of entries: {}", db.len());
//     // println!("Number of point count: {}", point);
//     // println!("Number of 4 anchors: {}", anchor.iter().filter(|(_, v)| *v == &4).count());
//     // println!("Number of 3 anchors: {}", anchor.iter().filter(|(_, v)| *v == &3).count());
//     // println!("Number of 2 anchors: {}", anchor.iter().filter(|(_, v)| *v == &2).count());
//     // println!("Number of 1 anchors: {}", anchor.iter().filter(|(_, v)| *v == &1).count());
//     // println!("Number of 0 anchors: {}", anchor.iter().filter(|(_, v)| *v == &0).count());
//     // println!("Time delta: {} ms", max - min);
//     // println!("Number of start time: {} ms", anchor.iter().fold(u64::MAX, |st, t| st.min(*t.0)));

//     drop(db);
//     println!("Time taken: {} s", (Instant::now() - start).as_secs());
//     Ok(())
// }

// fn to_db(buf: &mut [f32]) {
//     let mut ref_db = f32::MIN;
//     buf.iter().for_each(|v| ref_db = f32::max(ref_db, *v));

//     let amp_ref = ref_db * ref_db;
//     let offset = 10.0 * (f32::max(1e-10, amp_ref)).log10();
//     let mut log_spec_max = f32::MIN;

//     for val in buf.iter_mut() {
//         *val = 10.0 * (f32::max(1e-10, *val * *val)).log10() - offset;
//         log_spec_max = f32::max(log_spec_max, *val);
//     }

//     for val in buf.iter_mut() {
//         *val = f32::max(*val, log_spec_max - 80.0);
//     }
// }

// fn hz_to_mel(hz: f32) -> f32 {
//     2595.0 * (1.0 + hz / 700.0).log10()
// }

// fn mel_to_hz(mel: f32) -> f32 {
//     700.0 * (10f32.powf(mel / 2595.0) - 1.0)
// }

// fn mel_filterbank(
//     sr: u32,
//     n_fft: usize,
//     n_mels: usize,
//     fmin: f32,
//     fmax: f32,
// ) -> Vec<Vec<f32>> {
//     let mut filters = vec![vec![0.0; n_fft / 2 + 1]; n_mels];
//     let mel_min = hz_to_mel(fmin);
//     let mel_max = hz_to_mel(fmax);
//     let mel_points: Vec<f32> = (0..n_mels + 2)
//         .map(|i| mel_min + (mel_max - mel_min) * i as f32 / (n_mels + 1) as f32)
//         .collect();
//     println!("Mel points: {:?}", mel_points);
//     let hz_points: Vec<f32> = mel_points.into_iter().map(mel_to_hz).collect();
//     println!("Hz points: {:?}", hz_points);
//     let bin = |freq: f32| ((n_fft + 1) as f32 * freq / sr as f32).floor() as usize;

//     for m in 1..=n_mels {
//         let f_m_minus = bin(hz_points[m - 1]);
//         let f_m = bin(hz_points[m]);
//         let f_m_plus = bin(hz_points[m + 1]);
//         for k in f_m_minus..f_m {
//             filters[m - 1][k] = (k - f_m_minus) as f32 / (f_m - f_m_minus) as f32;
//         }
//         for k in f_m..f_m_plus {
//             filters[m - 1][k] = (f_m_plus - k) as f32 / (f_m_plus - f_m) as f32;
//         }
//     }
//     filters
// }

// fn mel_spectrogram(
//     stft: &[Vec<f32>],
//     sr: u32,
//     n_fft: usize,
//     n_mels: usize,
//     fmin: f32,
//     fmax: f32,
// ) -> Vec<Vec<f32>> {
//     let filters = mel_filterbank(sr, n_fft, n_mels, fmin, fmax);
//     stft.iter()
//         .map(|frame| {
//             filters
//                 .iter()
//                 .map(|filter| frame.iter().zip(filter).map(|(x, w)| x * w).sum())
//                 .collect()
//         })
//         .collect()
// }
