// Copyright 2023. The resback authors all rights reserved.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    headers::{authorization::Bearer, Authorization},
    response::IntoResponse,
    Extension, Json, TypedHeader,
};
use axum_extra::extract::CookieJar;
use axum_typed_multipart::TypedMultipart;

use crate::{
    jwt::authorize_user,
    mentoring::{
        order::MentoringOrder,
        schedule::{MentoringMethod, MentoringTime},
    },
    schema::{MentoringOrderCreationSchema, MentoringOrderSchema},
    user::{
        account::{validate_user_id, NormalUser, SeniorUser, User},
        UserType,
    },
    AppState, Error, Result,
};

pub async fn get_time_table(State(data): State<Arc<AppState>>) -> Result<impl IntoResponse> {
    Ok(Json(MentoringTime::get_all(&data.database).await?))
}

pub async fn create_mentoring_order(
    Extension(user): Extension<NormalUser>,
    State(data): State<Arc<AppState>>,
    TypedMultipart(order_data): TypedMultipart<MentoringOrderCreationSchema>,
) -> Result<impl IntoResponse> {
    let seller = SeniorUser::from_id(order_data.seller_id, &data.database).await?;

    if !seller.mentoring_status() {
        return Err(Error::InvalidRequestData {
            data: "senior user mentoring status".to_string(),
            expected: "true".to_string(),
            found: "false".to_string(),
        });
    }

    let time = MentoringTime::from_hour(order_data.time, &data.database).await?;
    let method = MentoringMethod::from_kind(seller.mentoring_method(), &data.database).await?;
    let order: MentoringOrderSchema = MentoringOrder::create(
        user.id(),
        seller.id(),
        &time,
        &method,
        seller.mentoring_price(),
        &order_data.content,
        &data.database,
    )
    .await?
    .into();

    Ok(Json(order))
}

pub async fn get_mentoring_order(
    Path(id): Path<u64>,
    cookie_jar: CookieJar,
    auth_header: Option<TypedHeader<Authorization<Bearer>>>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse> {
    let (user_type, user_id) =
        authorize_user(cookie_jar, auth_header, data.config.public_key.decoding_key()).await?;
    let order = MentoringOrder::from_id(id, &data.database).await?;

    match user_type {
        UserType::NormalUser => {
            validate_user_id(
                order.buyer().id(),
                &NormalUser::from_id(user_id, &data.database).await?,
            )?;
        }
        UserType::SeniorUser => match order.seller() {
            Some(seller) => {
                validate_user_id(
                    seller.id(),
                    &SeniorUser::from_id(user_id, &data.database).await?,
                )?;
            }
            None => Err(Error::Unauthorized)?,
        },
    }

    Ok(Json(MentoringOrderSchema::from(order)))
}
