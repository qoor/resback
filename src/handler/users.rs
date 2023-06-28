use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Json};
use axum_typed_multipart::TypedMultipart;

use crate::{schema::SeniorRegisterSchema, user::account::SeniorUser, AppState, Result};

pub async fn register_senior(
    State(data): State<Arc<AppState>>,
    TypedMultipart(register_data): TypedMultipart<SeniorRegisterSchema>,
) -> Result<impl IntoResponse> {
    let user_id = SeniorUser::register(&register_data, &data.database).await?;
    Ok(Json(serde_json::json!({ "id": user_id })))
}
