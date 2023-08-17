// Copyright 2023. The resback authors all rights reserved.

use axum::{async_trait, extract::multipart};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipartError};
use chrono::{DateTime, Utc};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tempfile::NamedTempFile;

use crate::{
    mentoring::{
        schedule::{MentoringSchedule, MentoringTime},
        MentoringMethodKind,
    },
    oauth::OAuthProvider,
    user::{account::UserId, UserType},
};

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

#[derive(Debug, Serialize, Deserialize, Clone, TryFromMultipart)]
pub struct UserIdentificationSchema {
    pub user_type: UserType,
    pub id: UserId,
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
    pub mentoring_price: u32,
    pub representative_careers: JsonArray<String>,
    pub description: String,
    pub email_verified: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SeniorSearchSchema {
    pub major: Option<String>,
    pub keyword: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct SeniorSearchResultSchema {
    pub seniors: Vec<SeniorUserInfoSchema>,
}

#[derive(TryFromMultipart)]
pub struct NormalUserUpdateSchema {
    pub nickname: String,
    pub picture: Option<FieldData<NamedTempFile>>,
}

#[derive(TryFromMultipart)]
pub struct SeniorUserUpdateSchema {
    pub nickname: String,
    pub picture: Option<FieldData<NamedTempFile>>,
    pub major: String,
    pub experience_years: i32,
    pub mentoring_price: i32,
    pub representative_careers: JsonArray<String>,
    pub description: String,
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
    async fn try_from_field(
        field: multipart::Field<'_>,
        _limit_bytes: Option<usize>,
    ) -> Result<Self, TypedMultipartError> {
        let field_name = field.name().unwrap_or("{unknown}").to_string();
        let field_text = field.text().await?;

        Ok(serde_json::from_str(&field_text).map_err(|_| TypedMultipartError::WrongFieldType {
            field_name,
            wanted_type: "JSON array".to_string(),
        })?)
    }
}

#[derive(Debug, Serialize)]
pub struct SeniorUserScheduleSchema {
    pub id: UserId,
    pub schedule: Vec<MentoringTime>,
    pub method: MentoringMethodKind,
    pub status: bool,
    pub always_on: bool,
}

impl From<MentoringSchedule> for SeniorUserScheduleSchema {
    fn from(value: MentoringSchedule) -> Self {
        Self {
            id: value.senior_id(),
            schedule: value.times().to_vec(),
            method: value.method(),
            status: value.status(),
            always_on: value.always_on(),
        }
    }
}

#[derive(TryFromMultipart, Debug)]
pub struct SeniorUserScheduleUpdateSchema {
    pub schedule: JsonArray<u32>,
    pub method: u32,
    pub status: bool,
    pub always_on: bool,
}

#[derive(Debug, Deserialize)]
pub struct EmailVerificationSchema {
    pub code: String,
}

#[derive(TryFromMultipart, Debug)]
pub struct MentoringOrderCreationSchema {
    pub seller_id: UserId,
    pub time: u32,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct MentoringOrderSchema {
    pub id: u64,
    pub buyer_id: UserId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seller_id: Option<UserId>,
    pub time: u32,
    pub method: MentoringMethodKind,
    pub price: u32,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct MentoringOrderListSchema {
    pub orders: Vec<MentoringOrderSchema>,
}

impl From<Vec<MentoringOrderSchema>> for MentoringOrderListSchema {
    fn from(value: Vec<MentoringOrderSchema>) -> Self {
        Self { orders: value }
    }
}
