# music_finder

Two vercel functions written in rust to search for spotify and youtube links in jiosavaan api.

## Usage

### Spotify

`POST /spotify`

* `body`: `{ url: string }`
* `response`: `TrackResult[]`

### Youtube

`POST /youtube`

* `body`: `{ url: string }`
* `response`: `TrackResult[]`

Where `TrackResult` is defined in `schema.rs` as:

### Jiosaavn API credits

* This application is deployed in cloudflare functions.
* Credit for the jiosaaavan api goes to [this repo](https://github.com/sumitkolhe/jiosaavn-api) with its [readme](https://github.com/sumitkolhe/jiosaavn-api#readme).


## Environment Variables

* `JIOSAVAN_URL`: Base URL of the jiosavaan API
* `SPOTIFY_CLIENT_ID`: Spotify client id
* `SPOTIFY_CLIENT_SECRET`: Spotify client secret
* `YOUTUBE_API_KEY`: Youtube API key
* `YOUTUBE_API_URL`: Youtube API base URL

## Deployment

This application is deployed in vercel functions.

## Local Development

To run this application locally, you need to have a vercel account and a vercel functions account.

## Dependencies

This application uses the following dependencies:

- [rust](https://www.rust-lang.org)
- [vercel](https://github.com/vercel/vercel)
- [vercel_runtime](https://github.com/vercel/vercel-rust)
- [reqwest](https://github.com/tauri-apps/reqwest)
- [serde_json](https://github.com/serde-rs/json)
- [serde](https://github.com/serde-rs/serde)
- [rspotify](https://github.com/RustAudio/rspotify)
- [regex](https://github.com/rust-lang/regex)
- [tokio](https://github.com/tokio-rs/tokio)
- [dotenvy](https://github.com/Geal/dotenvy)
- [tracing](https://github.com/tokio-rs/tracing)
- [tracing-subscriber](https://github.com/tokio-rs/tracing)
- [http](https://github.com/hyperium/http)