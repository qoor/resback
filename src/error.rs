// Copyright 2023. The resback authors all rights reserved.

use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub status: &'static str,
    pub message: String,
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

pub type Result<T> = std::result::Result<T, (StatusCode, ErrorResponse)>;
