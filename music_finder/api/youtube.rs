use music_finder::TrackSearch;
use regex::Regex;
use reqwest::Client;
use serde::Deserialize;
use serde_json::{Value, json};
use std::env;
use http::Method;
use vercel_runtime::{
    Body, Error, Request, RequestPayloadExt, Response, ServiceBuilder, StatusCode,
    http::bad_request, process_request, process_response, run_service, service_fn,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(false)
        .init();

    let handler = ServiceBuilder::new()
        .map_request(process_request)
        .map_response(process_response)
        .service(service_fn(get_youtube_song));

    run_service(handler).await
}

#[derive(Deserialize)]
pub struct Req {
    pub url: String,
}

pub async fn get_youtube_song(req: Request) -> Result<Response<Body>, Error> {
    tracing::info!("Received request: {:?}", req);
    
    if req.method() != Method::POST {
        return bad_request("Method not allowed only POST is allowed".to_string());
    }
    let url = match req.payload::<Req>() {
        Ok(Some(req)) => req.url,
        Ok(None) => return bad_request("Missing url".to_string()),
        Err(e) => return bad_request(e.to_string()),
    };
    let regex = Regex::new(r"youtube.*v[=/]([\d\w_-]+)|youtube.*e/([\d\w_-]+)|youtube.*embed/([\d\w_-]+)|youtu\.be/([\d\w_-]+)").unwrap();
    let client = Client::builder()
        .use_rustls_tls()
        .build()
        .map_err(|e| e.to_string())?;

    let jiosavan_url = env!("JIOSAVAN_URL");
    let youtube_api_key = env!("YOUTUBE_API_KEY");
    let youtube_api_url = env!("YOUTUBE_API_URL");
    let id = regex
        .captures(url.as_str())
        .and_then(|caps| caps.get(1))
        .ok_or_else(|| "Invalid URL".to_string())?;

    let response = client
        .get(format!("{}/videos", youtube_api_url))
        .query(&[
            ("part", "snippet"),
            ("id", id.as_str()),
            ("key", youtube_api_key.as_str()),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<Value>()
        .await
        .map_err(|e| e.to_string())?;

    let name = response["items"][0]["snippet"]["title"]
        .as_str()
        .ok_or("Failed to get track name".to_string())?;
    let response = client
        .get(format!("{}/api/search/songs", jiosavan_url))
        .query(&[("query", name)])
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let track_search = response
        .json::<TrackSearch>()
        .await
        .map_err(|e| e.to_string())?;

    if !track_search.success {
        return bad_request("Failed to search for track".to_string());
    }

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(json!(track_search.data.results.clone()).to_string().into())?)
}
