#![allow(unused)]

mod db;
mod music_finder;
mod schema;
mod utils;

use std::collections::HashMap;

use anyhow::Result;
use bincode::{Decode, Encode, config, encode_to_vec};
use db::*;
use redis::AsyncCommands;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Serialize, Deserialize)]
struct VideoInfo {
    id: String,
    title: String,
    thumbnail: String,
    duration: String,
    url: String,
    uploader: String,
    channel_url: String,
    description: String,
    timestamp: String,
    upload_date: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;
    let mut redis = connect_redis().await.expect("Failed to connect to Redis");
    let config = config::standard();

    println!("{:?}", get_all_songs(&mut redis).await?);
    Ok(())
}
