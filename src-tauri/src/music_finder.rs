use crate::schema::{Album, Artist, Artists, DownloadUrlItem, ImageItem, TrackList, TrackResult};
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub async fn find_jiosaavn_music(client: &Client, url: &str) -> Result<Vec<TrackResult>> {
    let jiosaavn_api_url = env!("JIOSAAVAN_API_URL");
    let res = client
        .get(format!("{}/api/songs", jiosaavn_api_url))
        .query(&[("link", url)])
        .send()
        .await?
        .json::<TrackList>()
        .await?;
    Ok(res.data)
}

pub async fn get_jiosaavan_url(val: TrackResult) -> String {
    let downloads = &val.download_url;
    if downloads.len() < 5 {
        downloads[downloads.len() - 1].url.clone()
    } else {
        downloads[4].url.clone()
    }
}

pub async fn download_jiosaavn_music(client: &Client, val: TrackResult) -> Result<Vec<u8>> {
    let url = get_jiosaavan_url(val).await;
    let res = client.get(url).send().await?.bytes().await?;
    Ok(res.to_vec())
}

pub async fn find_spotify_music(client: &Client, url: &str) -> Result<Vec<TrackResult>> {
    let music_finder_api_url = env!("MUSIC_FINDER_API_URL");
    let res = client
        .post(format!("{}/spotify", music_finder_api_url))
        .json(&json!({"url": url}))
        .send()
        .await?
        .json::<Vec<TrackResult>>()
        .await?;
    Ok(res)
}

pub async fn find_youtube_music(client: &Client, url: &str) -> Result<Vec<TrackResult>> {
    let music_finder_api_url = env!("MUSIC_FINDER_API_URL");
    let res = client
        .post(format!("{}/youtube", music_finder_api_url))
        .json(&json!({"url": url}))
        .send()
        .await?
        .json::<Vec<TrackResult>>()
        .await?;
    Ok(res)
}

pub async fn download_youtube_music(client: &Client, url: &str) -> Result<Vec<u8>> {
    let music_downloader_api_url = env!("MUSIC_DOWNLOADER_API_URL");
    let res = client
        .post(format!("{}/youtube", music_downloader_api_url))
        .json(&json!({"url": url}))
        .send()
        .await?
        .bytes()
        .await?;
    Ok(res.to_vec())
}

#[derive(Deserialize, Serialize, Debug)]
pub struct YoutubeMusicData {
    pub id: String,
    pub title: String,
    pub thumbnail: String,
    pub duration: String,
    pub url: String,
    pub uploader: String,
    pub channel_url: String,
    pub description: String,
    pub timestamp: String,
    pub upload_date: String,
}

impl YoutubeMusicData {
    pub fn to_track(self) -> TrackResult {
        TrackResult {
            id: self.id.clone(),
            url: self.url.clone(),
            name: self.title.clone(),
            duration: Some(self.duration.parse::<f64>().unwrap_or(0.0)),
            kind: "youtube".to_string(),
            year: None,
            release_date: None,
            label: None,
            explicit_content: false,
            play_count: None,
            language: "en".to_string(),
            has_lyrics: false,
            lyrics_id: None,
            copyright: None,
            album: Album {
                id: None,
                name: None,
                url: None,
            },
            artists: Artists {
                primary: vec![Artist {
                    id: self.id.clone(),
                    name: self.uploader.clone(),
                    role: "Artist".to_string(),
                    kind: "person".to_string(),
                    image: vec![],
                    url: self.channel_url.clone(),
                }],
                featured: vec![],
                all: vec![Artist {
                    id: self.id.clone(),
                    role: "Artist".to_string(),
                    kind: "person".to_string(),
                    image: vec![],
                    name: self.uploader.clone(),
                    url: self.channel_url.clone(),
                }],
            },
            image: vec![ImageItem {
                quality: "high".to_string(),
                url: self.thumbnail,
            }],
            download_url: vec![DownloadUrlItem {
                quality: "high".to_string(),
                url: self.url,
            }],
        }
    }
}

pub async fn get_youtube_music_data(client: &Client, url: &str) -> Result<YoutubeMusicData> {
    let music_downloader_api_url = env!("MUSIC_DOWNLOADER_API_URL");
    let res = client
        .get(format!("{}/youtube", music_downloader_api_url))
        .json(&json!({"url": url}))
        .send()
        .await?
        .error_for_status()?;
    println!("Response: {:#?}", res);
    let res = res.json::<YoutubeMusicData>().await?;
    Ok(res)
}
