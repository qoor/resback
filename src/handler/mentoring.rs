// Copyright 2023. The resback authors all rights reserved.

use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Json};

use crate::{user::mentoring::MentoringTime, AppState};

pub async fn get_time_table(State(data): State<Arc<AppState>>) -> crate::Result<impl IntoResponse> {
    Ok(Json(MentoringTime::get_all(&data.database).await?))
}
