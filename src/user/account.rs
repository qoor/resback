use std::sync::Arc;

use axum::{http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};

use crate::{oauth::OAuthProvider, AppState};

use super::OAuthUserData;

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize, Clone)]
pub struct NormalUser {
    id: u32,
    oauth_provider: OAuthProvider,
    oauth_id: String,
    nickname: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl NormalUser {
    pub async fn register_or_login(
        oauth_user: &OAuthUserData,
        state: &Arc<AppState>,
    ) -> Result<Self, (StatusCode, Json<serde_json::Value>)> {
        let login_response = Self::login(oauth_user, state).await;
        if let Ok(_) = login_response {
            return login_response;
        }

        Self::register(oauth_user, state).await
    }

    async fn register(
        oauth_user: &OAuthUserData,
        state: &Arc<AppState>,
    ) -> Result<Self, (StatusCode, Json<serde_json::Value>)> {
        sqlx::query!(
            "INSERT INTO normal_users (oauth_provider, oauth_id) VALUES (?, ?)",
            oauth_user.provider,
            oauth_user.id
        )
        .execute(&state.database)
        .await
        .map_err(|err| {
            let error_response = serde_json::json!({
                "status": "error",
                "message": format!("Database error: {}", err),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

        Self::login(oauth_user, state).await
    }

    async fn login(
        oauth_user: &OAuthUserData,
        state: &Arc<AppState>,
    ) -> Result<Self, (StatusCode, Json<serde_json::Value>)> {
        let user_data = sqlx::query_as_unchecked!(
            NormalUser,
            "SELECT * FROM normal_users WHERE oauth_provider = ? AND oauth_id = ?",
            oauth_user.provider,
            oauth_user.id
        )
        .fetch_optional(&state.database)
        .await
        .map_err(|err| {
            let error_response = serde_json::json!({
                "status": "error",
                "message": format!("Database error: {}", err),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?
        .ok_or({
            let error_response = serde_json::json!({
                "status": "fail",
                "message": "Invalid OAuth user data"
            });
            (StatusCode::BAD_REQUEST, Json(error_response))
        });

        user_data
    }
}

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize, Clone)]
pub struct SeniorUser {
    id: u32,
    email: String,
    password: String,
    name: String,
    phone: String,
    nickname: String,
    career_file_url: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}
