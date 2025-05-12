use anyhow::Result;
use reqwest::Client;
use serde_json::{Value, json};
use crate::schema::{TrackResult, TrackList};

pub async fn find_jiosaavn_music(client: Client, url: &str) -> Result<Vec<u8>> {
    let jiosaavn_api_url = std::env::var("JIOSAAVAN_API_URL")?;
    let res = client
        .get(format!("{}/api/songs", jiosaavn_api_url))
        .query(&[("link", url)])
        .send()
        .await?
        .json::<TrackList>()
        .await?;

    let downloads = &res.data[0].download_url;
    let url = if downloads.len() < 5 {
        &downloads[downloads.len() - 1].url
    } else {
        &downloads[4].url
    };
    let res = client.get(url).send().await?.bytes().await?;
    Ok(res.to_vec())
}

pub async fn download_jiosaavn_music(client: Client, val: TrackResult) -> Result<Vec<u8>> {
    let downloads = &val.download_url;
    let url = if downloads.len() < 5 {
        &downloads[downloads.len() - 1].url
    } else {
        &downloads[4].url
    };
    let res = client.get(url).send().await?.bytes().await?;
    Ok(res.to_vec())
}

pub async fn find_spotify_music(client: Client, url: &str) -> Result<Vec<TrackResult>> {
    let music_finder_api_url = std::env::var("MUSIC_FINDER_API_URL")?;
    let res = client
        .get(format!("{}/spotify", music_finder_api_url))
        .json(&json!({"url": url}))
        .send()
        .await?
        .json::<Vec<TrackResult>>()
        .await?;
    Ok(res)
}

pub async fn find_youtube_music(client: Client, url: &str) -> Result<Vec<TrackResult>> {
    let music_finder_api_url = std::env::var("MUSIC_FINDER_API_URL")?;
    let res = client
        .get(format!("{}/youtube", music_finder_api_url))
        .json(&json!({"url": url}))
        .send()
        .await?
        .json::<Vec<TrackResult>>()
        .await?;
    Ok(res)
}