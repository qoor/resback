// Copyright 2023. The resback authors all rights reserved.

use axum_typed_multipart::TryFromMultipart;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, TryFromMultipart)]
pub struct NormalLoginSchema {
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, TryFromMultipart)]
pub struct SeniorRegisterSchema {
    pub email: String,
    pub password: String,
    pub name: String,
    pub phone: String,
    pub major: String,
    pub experience_years: i32,
    pub mentoring_price: i32,
    pub representative_careers: Vec<String>,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, TryFromMultipart)]
pub struct SeniorLoginSchema {
    pub email: String,
    pub password: String,
}
