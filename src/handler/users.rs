use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};

use axum_typed_multipart::TypedMultipart;

use crate::{
    schema::SeniorRegisterSchema,
    user::account::{NormalUser, SeniorUser, User, UserId},
    AppState, Result,
};

pub async fn register_senior_user(
    State(data): State<Arc<AppState>>,
    TypedMultipart(register_data): TypedMultipart<SeniorRegisterSchema>,
) -> Result<impl IntoResponse> {
    let user_id = SeniorUser::register(&register_data, &data.database).await?;
    Ok(Json(serde_json::json!({ "id": user_id })))
}

pub async fn delete_normal_user(
    Path(id): Path<UserId>,
    State(data): State<Arc<AppState>>,
) -> crate::Result<impl IntoResponse> {
    NormalUser::delete(id, &data.database).await.map(|id| Json(serde_json::json!({ "uid": id })))
}

pub async fn delete_senior_user(
    Path(id): Path<UserId>,
    State(data): State<Arc<AppState>>,
) -> crate::Result<impl IntoResponse> {
    SeniorUser::delete(id, &data.database).await.map(|id| Json(serde_json::json!({ "uid": id })))
}
