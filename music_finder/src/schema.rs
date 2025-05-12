use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TrackSearch {
    pub success: bool,
    pub data: TrackSearchData,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TrackSearchData {
    pub total: i32,
    pub start: i32,
    pub results: Vec<TrackResult>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TrackResult {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub year: Option<String>,
    pub release_date: Option<String>,
    pub duration: Option<f64>,
    pub label: Option<String>,
    pub explicit_content: bool,
    pub play_count: Option<f64>,
    pub language: String,
    pub has_lyrics: bool,
    pub lyrics_id: Option<String>,
    pub url: String,
    pub copyright: Option<String>,
    pub album: Album,
    pub artists: Artists,
    pub image: Vec<ImageItem>,
    pub download_url: Vec<DownloadUrlItem>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Album {
    pub id: Option<String>,
    pub name: Option<String>,
    pub url: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Artists {
    pub primary: Vec<Artist>,
    pub featured: Vec<Artist>,
    pub all: Vec<Artist>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub role: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub image: Vec<ImageItem>,
    pub url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImageItem {
    pub quality: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DownloadUrlItem {
    pub quality: String,
    pub url: String,
}