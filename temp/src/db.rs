#![allow(unused)]

use std::collections::HashMap;

use anyhow::Result;
use redis::{
    AsyncCommands, Client, ClientTlsConfig, TlsCertificates, aio::ConnectionManager,
    from_redis_value,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, from_str, to_string};

use crate::schema::TrackResult;
use crate::utils::ANCHOR_POINTS;

// #[derive(Deserialize, Serialize, Debug)]
// struct SongInfo {
//     id: String,
//     title: String,
//     thumbnail: String,
//     duration: String,
//     url: String,
//     uploader: String,
//     channel_url: String,
//     description: String,
//     timestamp: String,
//     upload_date: String,
// }

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SongInfo {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub song_type: Option<String>,
    pub year: Option<String>,
    pub release_date: Option<String>,
    pub duration: Option<f64>,
    pub label: Option<String>,
    #[serde(rename = "explicitContent")]
    pub explicit_content: bool,
    #[serde(rename = "playCount")]
    pub play_count: Option<u64>,
    pub language: Option<String>,
    #[serde(rename = "hasLyrics")]
    pub has_lyrics: bool,
    #[serde(rename = "lyricsId")]
    pub lyrics_id: Option<String>,
    pub url: String,
    pub copyright: Option<String>,
    pub album: Option<Album>,
    pub artists: Option<Artists>,
    pub image: Vec<Image>,
    #[serde(rename = "downloadUrl")]
    pub download_url: Vec<DownloadUrl>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Album {
    pub id: Option<String>,
    pub name: Option<String>,
    pub url: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Artists {
    pub primary: Vec<Artist>,
    pub featured: Vec<Artist>,
    pub all: Vec<Artist>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Artist {
    pub id: Option<String>,
    pub name: Option<String>,
    pub role: Option<String>,
    #[serde(rename = "type")]
    pub artist_type: Option<String>,
    pub image: Vec<Image>,
    pub url: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Image {
    pub quality: String,
    pub url: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DownloadUrl {
    pub quality: String,
    pub url: String,
}

pub async fn connect_redis() -> Result<ConnectionManager> {
    let uri = env!("REDIS_URI");
    // let client = Client::open(uri)?;
    let client = Client::build_with_tls(
        uri,
        TlsCertificates {
            client_tls: None,
            root_cert: None,
        },
    )?;
    Ok(client.get_connection_manager().await?)
}

// -------------------------------------------------------

pub async fn get_redis_song(
    client: &mut ConnectionManager,
    key: u64,
) -> Result<Vec<(u64, String)>> {
    let val = client.smembers::<String, Vec<String>>(key.to_string()).await?;
    Ok(val
        .into_iter()
        .filter_map(|song| {
            song.split_once("|")
                .and_then(|(k, v)| Some((k.parse::<u64>().ok()?, v.to_string())))
        })
        .collect())
}

pub async fn set_redis_song(
    client: &mut ConnectionManager,
    key: u64,
    value: (u64, &str),
) -> Result<()> {
    let song = format!("{}|{}", value.0, value.1);
    Ok(client.sadd(key.to_string(), song).await?)
}

pub async fn delete_redis_song(client: &mut ConnectionManager, key: u64) -> Result<()> {
    Ok(client.del::<String, ()>(key.to_string()).await?)
}

// -----------------------------

pub async fn get_redis_json(client: &mut ConnectionManager, key: &str) -> Result<TrackResult> {
    Ok(from_str::<TrackResult>(
        &client
            .get::<String, String>(format!("song:{}", key))
            .await?,
    )?)
}

pub async fn set_redis_json(client: &mut ConnectionManager, value: TrackResult) -> Result<()> {
    Ok(client
        .set(format!("song:{}", value.id), to_string(&value)?)
        .await?)
}

pub async fn delete_redis_json(client: &mut ConnectionManager, key: &str) -> Result<()> {
    Ok(client.del(format!("song:{}", key)).await?)
}

pub async fn get_all_songs(client: &mut ConnectionManager) -> Result<Vec<TrackResult>> {
    let mut pipe = redis::pipe();

    let keys: Vec<String> = client.keys("song:*").await?;
    if keys.is_empty() {
        return Ok(Vec::new());
    }
    for key in &keys {
        pipe.get(key);
    }
    let values: Vec<Option<String>> = pipe
        .query_async(client)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get all songs: {:?}", e))?;
    let mut songs = Vec::with_capacity(keys.len());
    for (i, value) in values.into_iter().enumerate() {
        match value {
            Some(json_str) => match serde_json::from_str::<TrackResult>(&json_str) {
                Ok(song) => songs.push(song),
                Err(e) => eprintln!("Failed to parse song {}: {}", keys[i], e),
            },
            None => eprintln!("No value found for key: {}", keys[i]),
        }
    }
    Ok(songs)
}

// -------------------------------------------------------

pub async fn get_similar_songs(
    client: &mut ConnectionManager,
    keys: HashMap<u64, (u64, &str)>,
) -> Result<Vec<TrackResult>> {
    let mut anchors = HashMap::new();
    let mut song_times = HashMap::new();
    let mut main_song_id = "".to_string();

    for (key, (orig_time, orig_song_id)) in keys {
        main_song_id = orig_song_id.to_string();
        client
            .get::<String, Vec<String>>(key.to_string())
            .await?
            .into_iter()
            .filter_map(|v| {
                v.split_once("|")
                    .and_then(|(k, v)| Some((k.parse::<u64>().ok()?, v.to_string())))
            })
            .for_each(|(time, song_id)| {
                anchors
                    .entry(song_id.clone())
                    .and_modify(|count_map: &mut HashMap<u64, u8>| {
                        count_map
                            .entry(time)
                            .and_modify(|count| *count += 1)
                            .or_insert(1u8);
                    })
                    .or_insert(HashMap::new());
                song_times
                    .entry(song_id.clone())
                    .and_modify(|vec: &mut Vec<(u64, u64)>| vec.push((key, time)))
                    .or_insert(vec![(key, time)]);
                song_times
                    .entry(orig_song_id.to_string())
                    .and_modify(|vec: &mut Vec<(u64, u64)>| vec.push((key, orig_time)))
                    .or_insert(vec![(key, orig_time)]);
            });
    }
    let orig_song_time = song_times
        .get(&main_song_id)
        .ok_or(anyhow::anyhow!("No song time found"))?;

    let mut song_ids = anchors
        .into_iter()
        .filter_map(|(song_id, anchors)| {
            let count = anchors
                .iter()
                .filter(|(_, count)| **count as usize == ANCHOR_POINTS - 1)
                .count();

            if count < 10 {
                return None;
            }
            if let Some(song_time) = song_times.get(&song_id) {
                let time_diffs = song_time
                    .into_iter()
                    .zip(orig_song_time.iter())
                    .filter_map(|((k, time), (o_k, orig_time))| {
                        if k == o_k {
                            Some((*time, *orig_time))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<(u64, u64)>>();
                let mut time_diff = 0;
                time_diffs.windows(2).for_each(|window| {
                    let (time, orig_time) = window[0];
                    let (o_time, o_orig_time) = window[1];
                    let diff1 = (time as i64 - o_time as i64).abs();
                    let diff2 = (orig_time as i64 - o_orig_time as i64).abs();
                    time_diff += if (diff1 - diff2).abs() < 100 { 1 } else { 0 };
                });
                Some((time_diff, count, song_id))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    song_ids.sort_by_key(|(time_diff, count, _)| (*time_diff, *count));

    // println!("Keys: {:?}", keys);
    let mut songs = vec![];
    let song_ids = song_ids
        .into_iter()
        .rev()
        .take(5)
        .map(|(_, _, song_id)| song_id)
        .collect::<Vec<_>>();
    for song_id in song_ids {
        songs.push(get_redis_json(client, &song_id).await?);
    }
    Ok(songs)
}
