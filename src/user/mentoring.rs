// Copyright 2023. The resback authors all rights reserved.

use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::MySql;

use crate::{error::ErrorResponse, Result};

#[derive(sqlx::Type, Serialize, Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "snake_case")]
#[repr(u32)]
pub enum MentoringMethodKind {
    VoiceCall = 1,
    VideoCall = 2,
}

#[derive(sqlx::FromRow, Debug)]
pub struct MentoringTime {
    id: u64,
    hour: u32,
}

impl MentoringTime {
    pub async fn get_all(pool: &sqlx::Pool<MySql>) -> Result<Vec<Self>> {
        sqlx::query_as_unchecked!(Self, "SELECT * FROM mentoring_time")
            .fetch_all(pool)
            .await
            .map_err(|err| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse { status: "error", message: format!("Database error: {}", err) },
                )
            })
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn hour(&self) -> u32 {
        self.hour
    }
}

#[derive(sqlx::FromRow, Debug)]
pub struct MentoringMethod {
    kind: MentoringMethodKind,
    name: String,
}

impl MentoringMethod {
    pub async fn get_all(pool: &sqlx::Pool<MySql>) -> Result<Vec<Self>> {
        sqlx::query_as_unchecked!(Self, "SELECT id as kind, name FROM mentoring_method")
            .fetch_all(pool)
            .await
            .map_err(|err| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse { status: "error", message: format!("Database error: {}", err) },
                )
            })
    }

    pub fn kind(&self) -> MentoringMethodKind {
        self.kind
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
