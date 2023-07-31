// Copyright 2023. The resback authors all rights reserved.

use std::sync::Arc;

use aws_sdk_s3::primitives::ByteStream;
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use axum_typed_multipart::TypedMultipart;
use hyper::StatusCode;
use tokio::{fs, io};

use crate::{
    error::ErrorResponse,
    schema::{
        NormalUserInfoSchema, NormalUserUpdateSchema, SeniorRegisterSchema, SeniorSearchSchema,
        SeniorUserInfoSchema, SeniorUserScheduleSchema, SeniorUserScheduleUpdateSchema,
        SeniorUserUpdateSchema, UserIdentificationSchema,
    },
    user::{
        account::{NormalUser, NormalUserUpdate, SeniorUser, SeniorUserUpdate, User, UserId},
        mentoring::MentoringSchedule,
        UserType,
    },
    AppState, Result,
};

pub async fn register_senior_user(
    State(data): State<Arc<AppState>>,
    TypedMultipart(register_data): TypedMultipart<SeniorRegisterSchema>,
) -> Result<impl IntoResponse> {
    let id = SeniorUser::register(&register_data, &data.database).await?;
    Ok(Json(UserIdentificationSchema { user_type: UserType::SeniorUser, id }))
}

pub async fn get_senior_user_info(
    Path(id): Path<UserId>,
    State(data): State<Arc<AppState>>,
) -> crate::Result<impl IntoResponse> {
    let user = SeniorUser::from_id(id, &data.database).await?;
    Ok(Json(SeniorUserInfoSchema::from(user)))
}

pub async fn update_senior_user_profile(
    Path(id): Path<UserId>,
    State(data): State<Arc<AppState>>,
    TypedMultipart(update_data): TypedMultipart<SeniorUserUpdateSchema>,
) -> Result<impl IntoResponse> {
    let user = SeniorUser::from_id(id, &data.database).await?;

    let picture = match update_data.picture {
        Some(picture) => {
            let picture_dir = "uploaded-profile-image/senior";
            let temp_dir = std::env::temp_dir().join("senior");

            fs::create_dir(&temp_dir)
                .await
                .or_else(|error| match error.kind() {
                    io::ErrorKind::AlreadyExists => Ok(()),
                    _ => Err(error),
                })
                .map_err(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ErrorResponse {
                            status: "error",
                            message: "Failed to create temporary directory for the picture"
                                .to_string(),
                        },
                    )
                })?;

            let temp_file_path = temp_dir.join(id.to_string());
            let _temp_file =
                picture.contents.persist(&temp_file_path, true).await.map_err(|err| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ErrorResponse {
                            status: "error",
                            message: format!("Failed to receive picture file: {:?}", err),
                        },
                    )
                })?;

            let body = ByteStream::from_path(&temp_file_path).await.map_err(|err| {
                let _ = std::fs::remove_file(&temp_file_path);

                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse {
                        status: "error",
                        message: format!(
                            "Failed to get byte stream from temporary picture path: {:?}",
                            err
                        ),
                    },
                )
            })?;
            let _picture_upload_result = data
                .s3
                .put_object()
                .bucket(&data.config.s3_bucket)
                .key(format!("{}/{}", picture_dir, id))
                .body(body)
                .send()
                .await
                .map_err(|err| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ErrorResponse {
                            status: "error",
                            message: format!("Failed to upload profile picture: {:?}", err),
                        },
                    )
                })?;
            let _ = fs::remove_file(&temp_file_path).await;

            format!(
                "https://{}.s3.{}.amazonaws.com/{}/{}",
                data.config.s3_bucket, data.config.aws_region, picture_dir, id
            )
        }
        None => user.picture().to_string(),
    };
    let update_data = SeniorUserUpdate {
        nickname: update_data.nickname,
        picture,
        major: update_data.major,
        experience_years: update_data.experience_years,
        mentoring_price: update_data.mentoring_price,
        representative_careers: update_data.representative_careers,
        description: update_data.description,
    };
    user.update(&update_data, &data.database).await.map(|user| {
        Json(UserIdentificationSchema { user_type: UserType::SeniorUser, id: user.id() })
    })
}

pub async fn delete_senior_user(
    Path(id): Path<UserId>,
    State(data): State<Arc<AppState>>,
) -> crate::Result<impl IntoResponse> {
    SeniorUser::delete(id, &data.database)
        .await
        .map(|id| Json(UserIdentificationSchema { user_type: UserType::SeniorUser, id }))
}

pub async fn get_normal_user_info(
    Path(id): Path<UserId>,
    State(data): State<Arc<AppState>>,
) -> crate::Result<impl IntoResponse> {
    let user = NormalUser::from_id(id, &data.database).await?;
    Ok(Json(NormalUserInfoSchema::from(user)))
}

pub async fn update_normal_user_profile(
    Path(id): Path<UserId>,
    State(data): State<Arc<AppState>>,
    TypedMultipart(update_data): TypedMultipart<NormalUserUpdateSchema>,
) -> Result<impl IntoResponse> {
    let user = NormalUser::from_id(id, &data.database).await?;

    let picture = match update_data.picture {
        Some(picture) => {
            let picture_dir = "uploaded-profile-image/normal";
            let temp_dir = std::env::temp_dir().join("normal");

            fs::create_dir(&temp_dir)
                .await
                .or_else(|error| match error.kind() {
                    io::ErrorKind::AlreadyExists => Ok(()),
                    _ => Err(error),
                })
                .map_err(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ErrorResponse {
                            status: "error",
                            message: "Failed to create temporary directory for the picture"
                                .to_string(),
                        },
                    )
                })?;

            let temp_file_path = temp_dir.join(id.to_string());

            let _temp_file =
                picture.contents.persist(&temp_file_path, true).await.map_err(|err| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ErrorResponse {
                            status: "error",
                            message: format!("Failed to receive picture file: {:?}", err),
                        },
                    )
                })?;

            let body = ByteStream::from_path(&temp_file_path).await.map_err(|err| {
                let _ = std::fs::remove_file(&temp_file_path);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse {
                        status: "error",
                        message: format!(
                            "Failed to get byte stream from temporary picture path: {:?}",
                            err
                        ),
                    },
                )
            })?;
            let _picture_upload_result = data
                .s3
                .put_object()
                .bucket(&data.config.s3_bucket)
                .key(format!("{}/{}", picture_dir, id))
                .body(body)
                .send()
                .await
                .map_err(|err| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ErrorResponse {
                            status: "error",
                            message: format!("Failed to upload profile picture: {:?}", err),
                        },
                    )
                })?;
            let _ = fs::remove_file(&temp_file_path).await;

            format!(
                "https://{}.s3.{}.amazonaws.com/{}/{}",
                data.config.s3_bucket, data.config.aws_region, picture_dir, id
            )
        }
        None => user.picture().to_string(),
    };
    let update_data = NormalUserUpdate { nickname: update_data.nickname, picture };
    user.update(&update_data, &data.database).await.map(|user| {
        Json(UserIdentificationSchema { user_type: UserType::NormalUser, id: user.id() })
    })
}

pub async fn delete_normal_user(
    Path(id): Path<UserId>,
    State(data): State<Arc<AppState>>,
) -> crate::Result<impl IntoResponse> {
    NormalUser::delete(id, &data.database)
        .await
        .map(|id| Json(UserIdentificationSchema { user_type: UserType::NormalUser, id }))
}

pub async fn get_seniors(
    Query(search_info): Query<SeniorSearchSchema>,
    State(data): State<Arc<AppState>>,
) -> crate::Result<impl IntoResponse> {
    Ok(Json(SeniorUser::get_all(search_info, &data.database).await?))
}

pub async fn get_senior_mentoring_schedule(
    Path(id): Path<UserId>,
    State(data): State<Arc<AppState>>,
) -> crate::Result<impl IntoResponse> {
    let user = SeniorUser::from_id(id, &data.database).await?;
    let user_schedule: SeniorUserScheduleSchema =
        MentoringSchedule::from_senior_user(&user, &data.database)
            .await
            .map(|schedule| schedule.into())?;
    Ok(Json(user_schedule))
}

pub async fn update_senior_mentoring_schedule(
    Path(id): Path<UserId>,
    State(data): State<Arc<AppState>>,
    TypedMultipart(update_data): TypedMultipart<SeniorUserScheduleUpdateSchema>,
) -> crate::Result<impl IntoResponse> {
    let user = SeniorUser::from_id(update_data.id, &data.database).await?;
    let schedule = MentoringSchedule::from_senior_user(&user, &data.database).await?;
    Ok(Json(
        schedule
            .update(&update_data, &data.database)
            .await
            .map(|_| UserIdentificationSchema { user_type: UserType::SeniorUser, id })?,
    ))
}
