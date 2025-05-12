// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

mod db;
mod music_finder;
mod schema;
mod utils;

// use anyhow::Result;
use crate::db::*;
use crate::music_finder::*;
use crate::schema::*;
use crate::utils::*;
use anyhow::Result;
use redis::{AsyncCommands, aio::ConnectionManager};
use regex::Regex;
use tauri::{State, async_runtime::Runtime};
use tokio::runtime::Runtime as TokioRuntime;

struct AppState {
    pub redis_client: ConnectionManager,
    pub req_client: reqwest::Client,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    dotenvy::from_read(include_str!("../.env").as_bytes()).unwrap();

    let redis_client = Runtime::Tokio(TokioRuntime::new().unwrap())
        .block_on(connect_redis())
        .expect("Redis Connection Error");
    let req_client = reqwest::Client::builder()
        .use_rustls_tls()
        .build()
        .expect("Client Build Error");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            redis_client: redis_client.into(),
            req_client,
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            ping_redis_command,
            get_all_songs_command,
            get_song_from_url_command,
            add_music_to_db_command,
            add_youtube_music_to_db_command,
            delete_song_command,
            similar_songs_command,
            check_if_song_exists_command,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn ping_redis_command(state: State<'_, AppState>) -> Result<(), String> {
    let mut client = state.redis_client.clone();
    client
        .ping::<()>()
        .await
        .map_err(|e| format!("Redis Ping Error: {:?}", e))
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn get_all_songs_command(state: State<'_, AppState>) -> Result<Vec<TrackResult>, String> {
    let mut client = state.redis_client.clone();

    get_all_songs(&mut client)
        .await
        .map_err(|e| format!("Redis Get All Songs Error: {:?}", e))
}

#[tauri::command]
async fn delete_song_command(id: &str, state: State<'_, AppState>) -> Result<(), String> {
    let mut client = state.redis_client.clone();
    delete_song(&mut client, id)
        .await
        .map_err(|e| format!("Redis Delete Song Error: {:?}", e))
}

#[tauri::command]
async fn check_if_song_exists_command(
    id: &str,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let mut client = state.redis_client.clone();
    let exists = get_redis_json(&mut client, &format!("song:{}", id))
        .await
        .is_ok();
    Ok(exists)
}

#[tauri::command]
async fn get_song_from_url_command(
    url: &str,
    state: State<'_, AppState>,
) -> Result<Vec<TrackResult>, String> {
    let (y_reg, sp_reg, j_reg) = (
        Regex::new(r"youtu\.?be").map_err(|e| format!("Regex Error: {:?}", e))?,
        Regex::new(r"spotify").map_err(|e| format!("Regex Error: {:?}", e))?,
        Regex::new(r"jiosaavn").map_err(|e| format!("Regex Error: {:?}", e))?,
    );
    match (
        y_reg.is_match(url),
        sp_reg.is_match(url),
        j_reg.is_match(url),
    ) {
        (true, _, _) => find_youtube_music(&state.req_client, url)
            .await
            .map_err(|e| format!("Youtube Music Error: {:?}", e)),
        (_, true, _) => find_spotify_music(&state.req_client, url)
            .await
            .map_err(|e| format!("Spotify Music Error: {:?}", e)),
        (_, _, true) => find_jiosaavn_music(&state.req_client, url)
            .await
            .map_err(|e| format!("Jiosaavn Music Error: {:?}", e)),
        _ => Err("Invalid URL".to_string()),
    }
}

#[tauri::command]
async fn add_youtube_music_to_db_command(
    url: &str,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut redis_client = state.redis_client.clone();
    let music_data = get_youtube_music_data(&state.req_client, url)
        .await
        .map_err(|e| format!("Youtube Music Data Error: {:?}", e))?
        .to_track();
    let music = download_youtube_music(&state.req_client, url)
        .await
        .map_err(|e| format!("Youtube Music Error: {:?}", e))?;
    let id = music_data.id.clone();
    let mss = open_binary(music).map_err(|e| format!("Open Binary Error: {:?}", e))?;
    let (audio, sr) =
        extract_mono_audio(mss).map_err(|e| format!("Extract Mono Audio Error: {:?}", e))?;
    let (audio, _sr) = downsample(audio, sr, 2);
    let audio = normalise(audio);
    let stft = stft(audio, NUM_BINS, NUM_BINS / 2).map_err(|e| format!("STFT Error: {:?}", e))?;
    let filtered_stft = filter_stft(stft, sr as usize, NUM_BINS, NUM_BINS / 2);
    let data =
        filter_to_data(filtered_stft, &id).map_err(|e| format!("Filter To Data Error: {:?}", e))?;
    let mut songs = Vec::with_capacity(data.len());
    for (id, data) in data.into_iter() {
        songs.push((id, data.0, data.1));
    }
    set_all_songs(&mut redis_client, songs, music_data)
        .await
        .map_err(|e| format!("Redis Set All Songs Error: {:?}", e))?;
    Ok(())
}

#[tauri::command]
async fn add_music_to_db_command(
    val: TrackResult,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut redis_client = state.redis_client.clone();

    let id = val.id.clone();
    let music = download_jiosaavn_music(&state.req_client, val.clone())
        .await
        .map_err(|e| format!("Download Jiosaavn Music Error: {:?}", e))?;
    let mss = open_binary(music).map_err(|e| format!("Open Binary Error: {:?}", e))?;
    let (audio, sr) =
        extract_mono_audio(mss).map_err(|e| format!("Extract Mono Audio Error: {:?}", e))?;
    let (audio, _sr) = downsample(audio, sr, 2);
    let audio = normalise(audio);
    let stft = stft(audio, NUM_BINS, NUM_BINS / 2).map_err(|e| format!("STFT Error: {:?}", e))?;
    let filtered_stft = filter_stft(stft, sr as usize, NUM_BINS, NUM_BINS / 2);
    let data =
        filter_to_data(filtered_stft, &id).map_err(|e| format!("Filter To Data Error: {:?}", e))?;
    let mut songs = Vec::with_capacity(data.len());
    for (id, data) in data.into_iter() {
        songs.push((id, data.0, data.1));
    }
    set_all_songs(&mut redis_client, songs, val)
        .await
        .map_err(|e| format!("Redis Set All Songs Error: {:?}", e))?;
    Ok(())
}

#[tauri::command]
async fn similar_songs_command(
    audio: Vec<u8>,
    state: State<'_, AppState>,
) -> Result<Vec<TrackResult>, String> {
    let mut redis_client = state.redis_client.clone();

    let mss = crate::open_binary(audio).map_err(|e| format!("Open Binary Error: {:?}", e))?;
    let (audio, sr) =
        crate::extract_mono_audio(mss).map_err(|e| format!("Extract Mono Audio Error: {:?}", e))?;
    let (audio, _sr) = crate::downsample(audio, sr, 2);
    let audio = crate::normalise(audio);
    let stft = crate::stft(audio, crate::NUM_BINS, crate::NUM_BINS / 2)
        .map_err(|e| format!("STFT Error: {:?}", e))?;
    let filtered_stft = crate::filter_stft(stft, sr as usize, crate::NUM_BINS, crate::NUM_BINS / 2);
    let data = crate::filter_to_data(filtered_stft, "tmp")
        .map_err(|e| format!("Filter To Data Error: {:?}", e))?;
    let similar_songs = get_similar_songs(&mut redis_client, data)
        .await
        .map_err(|e| format!("Get Similar Songs Error: {:?}", e))?;
    Ok(similar_songs)
}
