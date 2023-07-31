// Copyright 2023. The resback authors all rights reserved.

use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::MySql;

use crate::{error::ErrorResponse, user::account::User, Result};

use super::account::{SeniorUser, UserId};

#[derive(sqlx::Type, Serialize, Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "snake_case")]
#[repr(u32)]
pub enum MentoringMethodKind {
    VoiceCall = 1,
    VideoCall = 2,
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
    times: Vec<MentoringTime>,
}

impl MentoringSchedule {
    pub async fn from_senior_user(
        senior_user: &SeniorUser,
        pool: &sqlx::Pool<MySql>,
    ) -> Result<Self> {
        MentoringScheduleRow::from_senior_user(senior_user, pool).await.map(|rows| Self {
            senior_id: senior_user.id(),
            times: rows.into_iter().map(|row| row.into()).collect(),
        })
    }

    pub fn senior_id(&self) -> UserId {
        self.senior_id
    }

    pub fn times(&self) -> &Vec<MentoringTime> {
        &self.times
    }
}
