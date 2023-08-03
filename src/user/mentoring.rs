// Copyright 2023. The resback authors all rights reserved.

use std::fmt;
use std::str::FromStr;

use axum::{async_trait, extract::multipart};
use axum_typed_multipart::TypedMultipartError;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::MySql;

use crate::{
    error::ErrorResponse, schema::SeniorUserScheduleUpdateSchema, user::account::User, Result,
};

use super::account::{SeniorUser, UserId};

#[derive(sqlx::Type, Serialize, Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "snake_case")]
#[repr(u32)]
pub enum MentoringMethodKind {
    VideoCall = 1,
    VoiceCall = 2,
}

impl fmt::Display for MentoringMethodKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match *self {
            MentoringMethodKind::VideoCall => "video_call",
            MentoringMethodKind::VoiceCall => "voice_call",
        };

        write!(f, "{}", s)
    }
}

impl FromStr for MentoringMethodKind {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "video_call" => Ok(MentoringMethodKind::VideoCall),
            "voice_call" => Ok(MentoringMethodKind::VoiceCall),
            _ => Err("Invalid mentoring method".to_string()),
        }
    }
}

#[async_trait]
impl axum_typed_multipart::TryFromField for MentoringMethodKind {
    async fn try_from_field(
        field: multipart::Field<'_>,
    ) -> std::result::Result<Self, TypedMultipartError> {
        let field_name = field.name().unwrap_or("{unknown}").to_string();
        let field_text = field.text().await?;

        MentoringMethodKind::from_str(&field_text).map_err(|_| {
            TypedMultipartError::WrongFieldType {
                field_name,
                wanted_type: "MentoringMethodKind".to_string(),
            }
        })
    }
}

#[derive(sqlx::FromRow, Serialize, Clone, Debug)]
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

    async fn new(id: u64, hour: u32) -> Self {
        Self { id, hour }
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

struct MentoringScheduleRow {
    id: u64,
    #[allow(dead_code)]
    senior_id: UserId,
    #[allow(dead_code)]
    time_id: u64,
    hour: u32,
}

impl MentoringScheduleRow {
    async fn from_senior_user(
        senior_user: &SeniorUser,
        pool: &sqlx::Pool<MySql>,
    ) -> Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT mentoring_schedule.*, mentoring_time.hour FROM mentoring_schedule INNER JOIN mentoring_time ON mentoring_time.id = time_id WHERE senior_id = ?",
            senior_user.id()
        )
        .fetch_all(pool)
        .await
        .map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse { status: "error", message: format!("Database error: {}", err) },
            )
        })
    }
}

impl From<MentoringScheduleRow> for MentoringTime {
    fn from(value: MentoringScheduleRow) -> Self {
        Self { id: value.id, hour: value.hour }
    }
}

pub struct MentoringSchedule {
    senior_id: UserId,
    schedule: Vec<MentoringTime>,
}

impl MentoringSchedule {
    pub async fn from_senior_user(
        senior_user: &SeniorUser,
        pool: &sqlx::Pool<MySql>,
    ) -> Result<Self> {
        MentoringScheduleRow::from_senior_user(senior_user, pool).await.map(|rows| Self {
            senior_id: senior_user.id(),
            schedule: rows.into_iter().map(|row| row.into()).collect(),
        })
    }

    pub async fn from_update_schema(
        senior_id: UserId,
        update_data: &SeniorUserScheduleUpdateSchema,
        pool: &sqlx::Pool<MySql>,
    ) -> Result<Self> {
        let schedule: Vec<MentoringTime> = MentoringTime::get_all(pool).await.map(|times| {
            times
                .into_iter()
                .filter_map(|time| match update_data.schedule.0.contains(&time.hour) {
                    true => Some(time),
                    false => None,
                })
                .collect()
        })?;

        Ok(Self { senior_id, schedule })
    }

    pub async fn update(
        self,
        update_data: &SeniorUserScheduleUpdateSchema,
        pool: &sqlx::Pool<MySql>,
    ) -> Result<Self> {
        let new_schedule = Self::from_update_schema(self.senior_id, update_data, pool).await?;
        let user = SeniorUser::from_id(self.senior_id, pool).await?;

        sqlx::query!("DELETE FROM mentoring_schedule WHERE senior_id = ?", user.id())
            .execute(pool)
            .await
            .map_err(|err| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse {
                        status: "error",
                        message: format!("Failed to delete previous user schedule: {}", err),
                    },
                )
            })?;

        for time in &new_schedule.schedule {
            sqlx::query!(
                "INSERT INTO mentoring_schedule (senior_id, time_id) VALUES (?, ?)",
                user.id(),
                time.id
            )
            .execute(pool)
            .await
            .map_err(|err| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse {
                        status: "error",
                        message: format!("Failed to insert new user schedule: {}", err),
                    },
                )
            })?;
        }

        Ok(new_schedule)
    }

    pub fn senior_id(&self) -> UserId {
        self.senior_id
    }

    pub fn times(&self) -> &Vec<MentoringTime> {
        &self.schedule
    }
}
