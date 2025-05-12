# Temp

This folder is a collection of rust code used to test the actual workings of the sonicscan music fingerprinting and identification using rust scripts and also visualizing using spectrograms.

## Usage

To run the code, simply run `cargo run` in the root of the project.

## Dependencies

This project uses the following dependencies:

- [anyhow](https://github.com/dtolnay/anyhow)
- [bincode](https://github.com/servo/bincode)
- [kdtree](https://github.com/Geal/kdtree)
- [rayon](https://github.com/rayon-rs/rayon)
- [realfft](https://github.com/Geal/realfft)
- [serde](https://github.com/serde-rs/serde)
- [sled](https://github.com/spacejam/sled)
- [symphonia](https://github.com/sonic-rs/symphonia)
- [tokio](https://github.com/tokio-rs/tokio)
- [image](https://github.com/image-rs/image)
- [hound](https://github.com/RustAudio/hound)
- [serde_json](https://github.com/serde-rs/json)
- [dotenvy](https://github.com/Geal/dotenvy)
- [reqwest](https://github.com/seanmonstar/reqwest)
- [redis](https://github.com/redis-rs/redis-rs)

## How I Build This

- One Day while watching youtube I found a [video](https://www.youtube.com/watch?v=a0CVCcb0RJM) by [Chigozirim](https://www.youtube.com/@cgzirim) about how shazam works and I was amazed by It. He build a music fingerprinting app using golang and it was amazing. He build it based on the blogpost by [Coding_Geek](https://drive.google.com/file/d/1ahyCTXBAZiuni6RTzHzLoOwwfTRFaU-C/view). So inspired by it I decided to build a music fingerprinting app using rust.

- I started by exploring the audio processing part and then I discovered [Symphonia](https://github.com/sonic-rs/symphonia) which is a rust crate for audio processing. First I build a function to extract mono audio from a file using it and kept a frequency limited to 41.1 KHz as it is the maximum frequency that the human ear can hear. I also implemented functions to downsample the audio and normalise it. Then I visualized the spectrogram of the audio using [image](https://github.com/image-rs/image).

- Then I started to build the fingerprinting part and I used the STFT to extract the frequency domain representation of the audio and used a Hann window function and a hop size of 512 and FFT size of 2048. Then I calculated the STFT for each band using [realfft](https://github.com/Geal/realfft) crate. 

- I implemented a function to filter the STFT based on the bands. To limit the number of bands based on which I compute the fingerprint so I used these bands `[
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
]` based on the human ear's frequency response by this [blogpost](https://unison.audio/eq-frequency-chart/) and this [blogpost](https://www.gear4music.com/blog/audio-frequency-range/). 

- Using the filtered STFT I computed the fingerprint using nearest neighbors of 5 peaks for each band and used the [kdtree](https://github.com/Geal/kdtree) crate and each of the fingerprint I computed the hash using the tuple values of the frequency and time and stored it in a valkey database.

- Then When I want to find the similar audio files I get the fingerprint of the audio file the functions and then search using the fingerprint as keys from the valkey database and then I sort the values based on the frequency, time difference and anchor count and then I return the top 3 respective songdata based on the filtered values with song_ids.
