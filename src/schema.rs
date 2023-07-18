// Copyright 2023. The resback authors all rights reserved.

use axum_typed_multipart::TryFromMultipart;
use serde::{Deserialize, Serialize};

use crate::user::account::UserId;

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

#[derive(Debug, Serialize, Clone)]
pub struct SeniorUserInfoSchema {
    pub id: UserId,
    pub nickname: String,
    pub major: String,
    pub experience_years: i32,
    pub mentoring_price: i32,
    pub representative_careers: Vec<String>,
    pub description: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CategorySearchSchema {
    pub major: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct CategorySearchResultSchema {
    pub seniors: Vec<SeniorUserInfoSchema>,
}
