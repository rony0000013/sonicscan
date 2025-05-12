# youtube_downloader

A simple music downloader api from youtube using yt-dlp.

## Usage

This api has two routes:

### GET /youtube

Takes a json payload with a single key `url` which is a youtube link.

Returns the data of the youtube video.

### POST /youtube

Takes a json payload with a single key `url` which is a youtube link.

Downloads the audio of the youtube video and returns it as a file.

## Deployment

This api is deployed using [fly.io](https://fly.io) with docker.

## Local Development

This api works best locally as youtube cookies cause problems in anonymous calls.

To run the api locally, simply run `uv run fastapi run dev` in the root of the project.

## Dependencies

This api uses the following dependencies:

- [fastapi](https://fastapi.tiangolo.com)
- [yt-dlp](https://github.com/yt-dlp/yt-dlp)
- [pydub](https://github.com/jiaaro/pydub)
