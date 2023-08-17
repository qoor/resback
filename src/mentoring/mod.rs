// Copyright 2023. The resback authors all rights reserved.

pub mod order;
pub mod schedule;

use core::fmt;
use std::str::FromStr;

use axum::{async_trait, extract::multipart};
use axum_typed_multipart::TypedMultipartError;
use serde::{Deserialize, Serialize};

use crate::error::BoxDynError;

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
    type Err = BoxDynError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "video_call" => Ok(MentoringMethodKind::VideoCall),
            "voice_call" => Ok(MentoringMethodKind::VoiceCall),
            _ => Err("Invalid mentoring method")?,
        }
    }
}

impl From<u32> for MentoringMethodKind {
    fn from(value: u32) -> Self {
        match value {
            2 => MentoringMethodKind::VoiceCall,
            _ => MentoringMethodKind::VideoCall,
        }
    }
}

#[async_trait]
impl axum_typed_multipart::TryFromField for MentoringMethodKind {
    async fn try_from_field(
        field: multipart::Field<'_>,
        _limit_bytes: Option<usize>,
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
