// Copyright 2023. The resback authors all rights reserved.

use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{async_trait, http::StatusCode};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sqlx::{
    types::chrono::{DateTime, Utc},
    MySql,
};

use crate::{
    error::ErrorResponse,
    nickname::{self, KoreanGenerator},
    schema::{JsonArray, NormalUserInfoSchema, SeniorRegisterSchema, SeniorUserInfoSchema},
};
use crate::{oauth::OAuthProvider, Result};

use super::OAuthUserData;

pub type UserId = u64;

const PEPPER: &str = "dV9h;TroC@ref}L}\\{_4d31.Fcv?ljN";

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

#[async_trait]
pub trait User: Sized {
    fn id(&self) -> UserId;

    fn refresh_token(&self) -> Option<&str>;

    async fn from_id(id: UserId, pool: &sqlx::Pool<MySql>) -> Result<Self>;

    async fn update_refresh_token(&self, token: &str, pool: &sqlx::Pool<MySql>) -> Result<&Self>;

    async fn delete(id: UserId, pool: &sqlx::Pool<MySql>) -> Result<UserId>;
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
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse { status: "error", message: format!("Database error: {}", err) },
            )
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
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse { status: "error", message: format!("Database error: {}", err) },
            )
        })?
        .ok_or((
            StatusCode::BAD_REQUEST,
            ErrorResponse { status: "fail", message: "Invalid OAuth user data".to_string() },
        ));

        user_data
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
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ErrorResponse {
                            status: "error",
                            message: format!("Database error: {}", err),
                        },
                    )
                })?
                .ok_or((
                    StatusCode::BAD_REQUEST,
                    ErrorResponse {
                        status: "fail",
                        message: "Invalid OAuth user data".to_string(),
                    },
                ));

        user_data
    }

    async fn update_refresh_token(&self, token: &str, pool: &sqlx::Pool<MySql>) -> Result<&Self> {
        sqlx::query!("UPDATE normal_users SET refresh_token = ? WHERE id = ?", token, self.id)
            .execute(pool)
            .await
            .map_err(|err| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse { status: "error", message: format!("Database error: {}", err) },
                )
            })?;

        Ok(&self)
    }

    async fn delete(id: UserId, pool: &sqlx::Pool<MySql>) -> Result<UserId> {
        let result = sqlx::query!("DELETE FROM normal_users WHERE id = ?", id)
            .execute(pool)
            .await
            .map_err(|err| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse { status: "error", message: format!("Database Error: {}", err) },
                )
            })?;

        match result.rows_affected() {
            1.. => Ok(id),
            _ => Err((
                StatusCode::NOT_FOUND,
                ErrorResponse { status: "fail", message: "Cannot find user".to_string() },
            )),
        }
    }
}

impl From<NormalUser> for NormalUserInfoSchema {
    fn from(value: NormalUser) -> Self {
        Self { id: value.id, oauth_provider: value.oauth_provider, nickname: value.nickname }
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
    major: String,
    experience_years: i32,
    mentoring_price: i32,
    representative_careers: String,
    description: String,
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
                ErrorResponse { status: "fail", message: "email or password is empty".to_string() },
            ));
        }

        let salt = SaltString::generate(&mut OsRng);
        let hashed_password = Argon2::new_with_secret(
            PEPPER.as_bytes(),
            argon2::Algorithm::default(),
            argon2::Version::default(),
            argon2::Params::default(),
        )
        .unwrap()
        .hash_password(register_data.password.as_bytes(), &salt)
        .map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    status: "error",
                    message: format!("Error while hashing password: {}", err),
                },
            )
        })
        .map(|hash| hash.to_string())?;

        let nickname = KoreanGenerator::new(nickname::Naming::Plain).next();
        let user = sqlx::query!(
            "INSERT INTO senior_users (email, password, name, phone, nickname, major, experience_years, mentoring_price, representative_careers, description) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            register_data.email,
            hashed_password,
            register_data.name,
            register_data.phone,
            nickname,
            register_data.major,
            register_data.experience_years,
            register_data.mentoring_price,
            register_data.representative_careers.to_string(),
            register_data.description
        ).execute(pool).await.map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, ErrorResponse {
            status: "error",
            message: format!("Database error: {}", err)
        }))?;

        Ok(user.last_insert_id())
    }

    pub async fn login(email: &str, password: &str, pool: &sqlx::Pool<MySql>) -> Result<Self> {
        if email.is_empty() || password.is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                ErrorResponse { status: "fail", message: "email or password is empty".to_string() },
            ));
        }

        let user =
            sqlx::query_as_unchecked!(Self, "SELECT * FROM senior_users WHERE email = ?", email)
                .fetch_optional(pool)
                .await
                .map_err(|err| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ErrorResponse {
                            status: "error",
                            message: format!("Database error: {}", err),
                        },
                    )
                })?
                .ok_or((
                    StatusCode::BAD_REQUEST,
                    ErrorResponse {
                        status: "fail",
                        message: "Invalid email or password".to_string(),
                    },
                ))?;

        let password_verified = match PasswordHash::new(&user.password) {
            Ok(parsed_hash) => Argon2::new_with_secret(
                PEPPER.as_bytes(),
                argon2::Algorithm::default(),
                argon2::Version::default(),
                argon2::Params::default(),
            )
            .unwrap()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_or(false, |_| true),
            Err(_) => false,
        };

        if !password_verified {
            return Err((
                StatusCode::BAD_REQUEST,
                ErrorResponse { status: "fail", message: "Invalid email or password".to_string() },
            ));
        }

        Ok(user)
    }

    pub fn email(&self) -> &str {
        &self.email
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
                        ErrorResponse {
                            status: "error",
                            message: format!("Database error: {}", err),
                        },
                    )
                })?
                .ok_or((
                    StatusCode::BAD_REQUEST,
                    ErrorResponse { status: "fail", message: "Invalid senior user id".to_string() },
                ));

        user_data
    }

    async fn update_refresh_token(&self, token: &str, pool: &sqlx::Pool<MySql>) -> Result<&Self> {
        sqlx::query!("UPDATE senior_users SET refresh_token = ? WHERE id = ?", token, self.id)
            .execute(pool)
            .await
            .map_err(|err| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse { status: "error", message: format!("Database error: {}", err) },
                )
            })?;

        Ok(&self)
    }

    async fn delete(id: UserId, pool: &sqlx::Pool<MySql>) -> Result<UserId> {
        let result = sqlx::query!("DELETE FROM senior_users WHERE id = ?", id)
            .execute(pool)
            .await
            .map_err(|err| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse { status: "error", message: format!("Database Error: {}", err) },
                )
            })?;

        match result.rows_affected() {
            1.. => Ok(id),
            _ => Err((
                StatusCode::NOT_FOUND,
                ErrorResponse { status: "fail", message: "Cannot find user".to_string() },
            )),
        }
    }
}

impl From<SeniorUser> for SeniorUserInfoSchema {
    fn from(value: SeniorUser) -> Self {
        SeniorUserInfoSchema {
            id: value.id,
            nickname: value.nickname,
            major: value.major,
            experience_years: value.experience_years,
            mentoring_price: value.mentoring_price,
            representative_careers: JsonArray::from_str(&value.representative_careers)
                .unwrap_or_default(),
            description: value.description,
        }
    }
}

pub async fn get_seniors_from_major(
    major: &str,
    pool: &sqlx::Pool<MySql>,
) -> Result<Vec<SeniorUserInfoSchema>> {
    let seniors: Vec<SeniorUserInfoSchema> =
        sqlx::query_as_unchecked!(SeniorUser, "SELECT * FROM senior_users WHERE major = ?", major)
            .fetch_all(pool)
            .await
            .map_err(|err| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse {
                        status: "error",
                        message: format!("Database error: {:?}", err),
                    },
                )
            })?
            .into_iter()
            .map(|senior| senior.into())
            .collect();

    Ok(seniors)
}
