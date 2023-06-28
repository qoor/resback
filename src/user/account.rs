// Copyright 2023. The resback authors all rights reserved.

use std::str::FromStr;

use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{async_trait, http::StatusCode, Json};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sqlx::{
    types::chrono::{DateTime, Utc},
    MySql,
};

use crate::{
    error::ErrorResponse,
    nickname::{self, KoreanGenerator},
    schema::SeniorRegisterSchema,
};
use crate::{oauth::OAuthProvider, Result};

use super::OAuthUserData;

pub type UserId = u64;

const FRONT_PEPPER: &str = "dV9h;TroC@ref}L}\\{_4d31.Fcv?ljN";
const BACK_PEPPER: &str = "s!\\uf@99E95K1B[]P91H{U\"SgI}*Id!";

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
    fn id(&self) -> UserId;

    fn refresh_token(&self) -> Option<&str>;

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

    pub async fn from_oauth_user(
        oauth_user: &OAuthUserData,
        pool: &sqlx::Pool<MySql>,
    ) -> Result<Self> {
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
        &self,
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
    fn id(&self) -> UserId {
        self.id
    }

    fn refresh_token(&self) -> Option<&str> {
        self.refresh_token.as_deref()
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
    refresh_token: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl SeniorUser {
    pub async fn register(
        register_data: &SeniorRegisterSchema,
        pool: &sqlx::Pool<MySql>,
    ) -> Result<UserId> {
        if register_data.email.is_empty() || register_data.password.is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    status: "fail",
                    message: "email or password is empty".to_string(),
                }),
            ));
        }

        let salt = SaltString::generate(&mut OsRng);
        let password = String::new() + FRONT_PEPPER + &register_data.password + BACK_PEPPER;
        let hashed_password = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|err| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        status: "error",
                        message: format!("Error while hashing password: {}", err),
                    }),
                )
            })
            .map(|hash| hash.to_string())?;

        let user = sqlx::query!(
            "INSERT INTO senior_users (email, password, name, phone, career_file_url) VALUES (?, ?, ?, ?, ?)",
            register_data.email,
            hashed_password,
            register_data.name,
            register_data.phone,
            register_data.career_file_url
        ).execute(pool).await.map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
            status: "error",
            message: format!("Database error: {}", err)
        })))?;

        Ok(user.last_insert_id())
    }

    pub async fn login(email: &str, password: &str, pool: &sqlx::Pool<MySql>) -> Result<Self> {
        if email.is_empty() || password.is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    status: "fail",
                    message: "email or password is empty".to_string(),
                }),
            ));
        }

        let user =
            sqlx::query_as_unchecked!(Self, "SELECT * FROM senior_users WHERE email = ?", email)
                .fetch_optional(pool)
                .await
                .map_err(|err| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            status: "error",
                            message: format!("Database error: {}", err),
                        }),
                    )
                })?
                .ok_or((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        status: "fail",
                        message: "Invalid email or password".to_string(),
                    }),
                ))?;

        let password = String::new() + FRONT_PEPPER + password + BACK_PEPPER;
        let password_verified = match PasswordHash::new(&user.password) {
            Ok(parsed_hash) => Argon2::default()
                .verify_password(password.as_bytes(), &parsed_hash)
                .map_or(false, |_| true),
            Err(_) => false,
        };

        if !password_verified {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    status: "fail",
                    message: "Invalid email or password".to_string(),
                }),
            ));
        }

        Ok(user)
    }
}

#[async_trait]
impl User for SeniorUser {
    fn id(&self) -> UserId {
        self.id
    }

    fn refresh_token(&self) -> Option<&str> {
        self.refresh_token.as_deref()
    }

    async fn from_id(id: UserId, pool: &sqlx::Pool<MySql>) -> Result<Self> {
        let user_data =
            sqlx::query_as_unchecked!(Self, "SELECT * FROM senior_users WHERE id = ?", id)
                .fetch_optional(pool)
                .await
                .map_err(|err| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            status: "error",
                            message: format!("Database error: {}", err),
                        }),
                    )
                })?
                .ok_or((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        status: "fail",
                        message: "Invalid senior user id".to_string(),
                    }),
                ));

        user_data
    }
}
