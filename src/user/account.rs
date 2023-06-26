// Copyright 2023. The resback authors all rights reserved.

use std::str::FromStr;

use axum::{async_trait, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::{
    types::chrono::{DateTime, Utc},
    MySql,
};

use crate::{
    error::ErrorResponse,
    nickname::{self, KoreanGenerator},
};
use crate::{oauth::OAuthProvider, Result};

use super::OAuthUserData;

pub type UserId = u64;

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize, Clone)]
pub struct NormalUser {
    id: UserId,
    oauth_provider: OAuthProvider,
    oauth_id: String,
    nickname: String,
    refresh_token: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub enum UserType {
    NormalUser,
    SeniorUser,
}

impl FromStr for UserType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "NormalUser" => Ok(Self::NormalUser),
            "SeniorUser" => Ok(Self::SeniorUser),
            _ => Err("Invalid user type string".to_string()),
        }
    }
}

impl std::fmt::Display for UserType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[async_trait]
pub trait User: Sized {
    fn id(self: &Self) -> UserId;

    async fn from_id(id: UserId, pool: &sqlx::Pool<MySql>) -> Result<Self>;
}

impl NormalUser {
    pub async fn register(oauth_user: &OAuthUserData, pool: &sqlx::Pool<MySql>) -> Result<UserId> {
        let nickname = KoreanGenerator::new(nickname::Naming::Plain).next();
        let result = sqlx::query!(
            "INSERT INTO normal_users (oauth_provider, oauth_id, nickname) VALUES (?, ?, ?)",
            oauth_user.provider,
            oauth_user.id,
            nickname
        )
        .execute(pool)
        .await
        .map_err(|err| {
            let error_response =
                ErrorResponse { status: "error", message: format!("Database error: {}", err) };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

        Ok(result.last_insert_id())
    }

    pub async fn login(oauth_user: &OAuthUserData, pool: &sqlx::Pool<MySql>) -> Result<Self> {
        let user_data = sqlx::query_as_unchecked!(
            Self,
            "SELECT * FROM normal_users WHERE oauth_provider = ? AND oauth_id = ?",
            oauth_user.provider(),
            oauth_user.id()
        )
        .fetch_optional(pool)
        .await
        .map_err(|err| {
            let error_response =
                ErrorResponse { status: "error", message: format!("Database error: {}", err) };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?
        .ok_or({
            let error_response =
                ErrorResponse { status: "fail", message: "Invalid OAuth user data".to_string() };
            (StatusCode::BAD_REQUEST, Json(error_response))
        });

        user_data
    }

    pub async fn update_refresh_token(
        self: &Self,
        token: &str,
        pool: &sqlx::Pool<MySql>,
    ) -> Result<&Self> {
        sqlx::query!("UPDATE normal_users SET refresh_token = ? WHERE id = ?", token, self.id)
            .execute(pool)
            .await
            .map_err(|err| {
                let error_response =
                    ErrorResponse { status: "error", message: format!("Database error: {}", err) };
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
            })?;

        Ok(&self)
    }
}

#[async_trait]
impl User for NormalUser {
    fn id(self: &Self) -> UserId {
        self.id
    }

    async fn from_id(id: UserId, pool: &sqlx::Pool<MySql>) -> Result<Self> {
        let user_data =
            sqlx::query_as_unchecked!(Self, "SELECT * FROM normal_users WHERE id = ?", id)
                .fetch_optional(pool)
                .await
                .map_err(|err| {
                    let error_response = ErrorResponse {
                        status: "error",
                        message: format!("Database error: {}", err),
                    };
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
                })?
                .ok_or({
                    let error_response = ErrorResponse {
                        status: "fail",
                        message: "Invalid OAuth user data".to_string(),
                    };
                    (StatusCode::BAD_REQUEST, Json(error_response))
                });

        user_data
    }
}

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize, Clone)]
pub struct SeniorUser {
    id: UserId,
    email: String,
    password: String,
    name: String,
    phone: String,
    nickname: String,
    career_file_url: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}
