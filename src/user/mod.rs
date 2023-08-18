// Copyright 2023. The resback authors all rights reserved.

use std::str::FromStr;

use axum::{async_trait, extract::multipart};
use axum_typed_multipart::TypedMultipartError;
use serde::{Deserialize, Serialize};

use crate::{error::BoxDynError, oauth::OAuthProvider};

pub mod account;
pub mod picture;

mod nickname;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum UserType {
    NormalUser,
    SeniorUser,
}

impl FromStr for UserType {
    type Err = BoxDynError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "NormalUser" => Ok(Self::NormalUser),
            "SeniorUser" => Ok(Self::SeniorUser),
            _ => Err("Invalid user type string")?,
        }
    }
}

impl std::fmt::Display for UserType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[async_trait]
impl axum_typed_multipart::TryFromField for UserType {
    async fn try_from_field(
        field: multipart::Field<'_>,
        _limit_bytes: Option<usize>,
    ) -> Result<Self, TypedMultipartError> {
        let field_name = field.name().unwrap_or("{unknown}").to_string();
        let field_text = field.text().await?;

        Ok(UserType::from_str(&field_text).map_err(|_| TypedMultipartError::WrongFieldType {
            field_name,
            wanted_type: "JSON array".to_string(),
        })?)
    }
}

#[derive(Debug)]
pub struct OAuthUserData {
    provider: OAuthProvider,
    id: String,
}

impl OAuthUserData {
    pub fn new(provider: OAuthProvider, id: &str) -> Self {
        Self { provider, id: id.to_string() }
    }
    pub fn provider(&self) -> OAuthProvider {
        self.provider
    }
    pub fn id(&self) -> &str {
        &self.id
    }
}
