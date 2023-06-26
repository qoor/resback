// Copyright 2023. The resback authors all rights reserved.

use axum::{http::StatusCode, Json};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub status: &'static str,
    pub message: String,
}

pub type Result<T> = std::result::Result<T, (StatusCode, Json<ErrorResponse>)>;
