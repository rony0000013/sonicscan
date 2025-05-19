use music_finder::TrackSearch;
use regex::Regex;
use reqwest::Client;
use rspotify::{ClientCredsSpotify, Credentials, model::idtypes::TrackId, prelude::BaseClient};
use serde::Deserialize;
use serde_json::json;
use http::Method;
use vercel_runtime::{
    Body, Error, Request, RequestPayloadExt, Response, ServiceBuilder, StatusCode,
    http::bad_request, process_request, process_response, run_service, service_fn,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(false)
        .init();

    let handler = ServiceBuilder::new()
        .map_request(process_request)
        .map_response(process_response)
        .service(service_fn(get_spotify_song));

    run_service(handler).await
}

#[derive(Deserialize)]
pub struct Req {
    pub url: String,
}

pub async fn get_spotify_song(req: Request) -> Result<Response<Body>, Error> {
    tracing::info!("Received request: {:?}", req);
    
    if req.method() != Method::POST {
        return bad_request("Method not allowed only POST is allowed".to_string());
    }
    let url = match req.payload::<Req>() {
        Ok(Some(req)) => req.url,
        Ok(None) => return bad_request("Missing url".to_string()),
        Err(e) => return bad_request(e.to_string()),
    };
    let regex = Regex::new(r"spotify.+/track/([\w\d]+)").unwrap();
    let client = Client::builder()
        .use_rustls_tls()
        .build()
        .map_err(|e| e.to_string())?;

    let jiosavan_url = env!("JIOSAVAN_URL");
    let spotify_client_id = env!("SPOTIFY_CLIENT_ID");
    let spotify_client_secret = env!("SPOTIFY_CLIENT_SECRET");

    let creds = Credentials::new(spotify_client_id.as_str(), spotify_client_secret.as_str());
    tracing::info!("Credentials: {:?}", creds);
    let spotify = ClientCredsSpotify::new(creds);
    tracing::info!("Spotify: {:?}", spotify);

    spotify.request_token().await.map_err(|e| format!("Failed to get token: {}", e))?;

    let id = regex
        .captures(url.as_str())
        .and_then(|caps| caps.get(1))
        .ok_or_else(|| "Invalid URL".to_string())?;

    let track_id = TrackId::from_id(id.as_str()).map_err(|e| e.to_string())?;
    let track = spotify
        .track(track_id, None)
        .await
        .map_err(|e| e.to_string())?;

    let name = if track.artists.is_empty() {
        track.name.clone()
    } else {
        format!("{} {}", track.name, track.artists[0].name)
    };
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
