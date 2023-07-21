// Copyright 2023. The resback authors all rights reserved.

use axum::{async_trait, extract::multipart};
use axum_typed_multipart::{TryFromMultipart, TypedMultipartError};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{oauth::OAuthProvider, user::account::UserId};

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
    pub representative_careers: JsonArray<String>,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, TryFromMultipart)]
pub struct SeniorLoginSchema {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct NormalUserInfoSchema {
    pub id: UserId,
    pub oauth_provider: OAuthProvider,
    pub nickname: String,
    pub picture: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct SeniorUserInfoSchema {
    pub id: UserId,
    pub nickname: String,
    pub picture: String,
    pub major: String,
    pub experience_years: i32,
    pub mentoring_price: i32,
    pub representative_careers: JsonArray<String>,
    pub description: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SeniorSearchSchema {
    pub major: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct SeniorSearchResultSchema {
    pub seniors: Vec<SeniorUserInfoSchema>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JsonArray<T>(pub Vec<T>);

impl<T> Default for JsonArray<T> {
    fn default() -> Self {
        Self(Vec::<T>::default())
    }
}

impl<T: DeserializeOwned> JsonArray<T> {
    pub fn from_str(s: &str) -> Result<Self, serde_json::Error> {
        Ok(Self(serde_json::from_str(s)?))
    }
}

impl<T: Serialize> std::fmt::Display for JsonArray<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).map_err(|_| std::fmt::Error)?)
    }
}

#[async_trait]
impl<T: DeserializeOwned> axum_typed_multipart::TryFromField for JsonArray<T> {
    async fn try_from_field(field: multipart::Field<'_>) -> Result<Self, TypedMultipartError> {
        let field_name = field.name().unwrap_or("{unknown}").to_string();
        let field_text = field.text().await?;

        Ok(serde_json::from_str(&field_text).map_err(|_| TypedMultipartError::WrongFieldType {
            field_name,
            wanted_type: "JSON array".to_string(),
        })?)
    }
}
