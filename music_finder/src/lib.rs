pub mod schema;

// Re-export the schema types for easier access
pub use schema::*;


// use schema::*;

// use anyhow::Result;
// use axum::{
//     Json, Router,
//     body::Body,
//     extract::State,
//     http::{Response, status::StatusCode},
//     response::IntoResponse,
//     routing::{get, post},
// };
// use regex::Regex;
// use reqwest::Client;
// use rspotify::{ClientCredsSpotify, Credentials, model::idtypes::TrackId, prelude::BaseClient};
// use serde::{Deserialize, Serialize};
// use serde_json::Value;



// #[derive(Deserialize)]
// pub struct Req {
//     pub url: String,
// }



// #[derive(Serialize)]
// struct ErrorResponse {
//     error: String,
// }

// pub struct MyError(pub String);

// impl IntoResponse for MyError {
//     fn into_response(self) -> Response<Body> {
//         let body = Json(ErrorResponse { error: self.0 });
//         (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
//     }
// }

// fn to_error(e: impl std::fmt::Display) -> MyError {
//     MyError(e.to_string())
// }
