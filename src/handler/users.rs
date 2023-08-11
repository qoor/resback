// Copyright 2023. The resback authors all rights reserved.

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use axum_typed_multipart::TypedMultipart;
use tokio::{fs, io};

use crate::{
    error::Error,
    schema::{
        EmailVerificationSchema, NormalUserInfoSchema, NormalUserUpdateSchema,
        SeniorRegisterSchema, SeniorSearchSchema, SeniorUserInfoSchema, SeniorUserScheduleSchema,
        SeniorUserScheduleUpdateSchema, SeniorUserUpdateSchema, UserIdentificationSchema,
    },
    user::{
        account::{NormalUser, NormalUserUpdate, SeniorUser, SeniorUserUpdate, User, UserId},
        mentoring::{MentoringMethodKind, MentoringSchedule},
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

    let picture_url = match update_data.picture {
        Some(picture) => {
            let (temp_path, path_to_push) =
                get_user_picture_paths(&UserType::SeniorUser, &id).await?;

            picture.contents.persist(&temp_path).map_err(|err| Error::PersistFile {
                path: temp_path.to_path_buf(),
                source: err.into(),
            })?;

            data.s3.push_file(&temp_path, &path_to_push).await?
        }
        None => user.picture().to_string(),
    };

    let update_data = SeniorUserUpdate {
        nickname: update_data.nickname,
        picture: picture_url,
        major: update_data.major,
        experience_years: update_data.experience_years,
        mentoring_price: update_data.mentoring_price,
        representative_careers: update_data.representative_careers,
        description: update_data.description,
    };

    user.update_profile(&update_data, &data.database).await.map(|user| {
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

    let picture_url = match update_data.picture {
        Some(picture) => {
            let (temp_path, path_to_push) =
                get_user_picture_paths(&UserType::NormalUser, &id).await?;

            picture.contents.persist(&temp_path).map_err(|err| Error::PersistFile {
                path: temp_path.to_path_buf(),
                source: err.into(),
            })?;

            data.s3.push_file(&temp_path, &path_to_push).await?
        }
        None => user.picture().to_string(),
    };

    let update_data = NormalUserUpdate { nickname: update_data.nickname, picture: picture_url };

    user.update_profile(&update_data, &data.database).await.map(|user| {
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
    let user = SeniorUser::from_id(id, &data.database).await?;
    let schedule = MentoringSchedule::from_senior_user(&user, &data.database).await?;
    let method: MentoringMethodKind = update_data.method.try_into().map_err(Error::Unhandled)?;

    schedule.update(&update_data, &data.database).await?;
    user.update_mentoring_data(&method, update_data.status, update_data.always_on, &data.database)
        .await?;

    Ok(Json(UserIdentificationSchema { user_type: UserType::SeniorUser, id }))
}

pub async fn register_senior_user_verification(
    Path(id): Path<UserId>,
    State(data): State<Arc<AppState>>,
) -> crate::Result<impl IntoResponse> {
    let user = SeniorUser::from_id(id, &data.database).await?;
    let verification_code = user.register_verification(&data.database).await?;

    data.ses
        .send_mail(
            "no-reply@respec.team",
            user.email(),
            "respec.team 가입을 위한 인증 코드입니다.",
            &format!(
                "안녕하세요, respec.team입니다.
계정 가입을 완료하기 위한 인증 코드는 다음과 같습니다.

{}

저희 서비스에 가입해 주셔서 진심으로 감사드립니다.",
                verification_code
            ),
        )
        .await?;

    Ok(Json(UserIdentificationSchema { user_type: UserType::SeniorUser, id }))
}

pub async fn verify_senior_user(
    Path(id): Path<UserId>,
    Query(payload): Query<EmailVerificationSchema>,
    State(data): State<Arc<AppState>>,
) -> crate::Result<impl IntoResponse> {
    let user = SeniorUser::from_id(id, &data.database).await?;

    user.verify_email(&payload.code, &data.database)
        .await
        .map(|_| Json(UserIdentificationSchema { user_type: UserType::SeniorUser, id }))
}

async fn get_user_picture_paths(
    user_type: &UserType,
    id: &UserId,
) -> crate::Result<(std::path::PathBuf, String)> {
    let user_type_str = match user_type {
        UserType::NormalUser => "normal",
        UserType::SeniorUser => "senior",
    };
    let temp_dir = std::env::temp_dir().join("respec.team").join(user_type_str);

    fs::create_dir_all(&temp_dir)
        .await
        .or_else(|error| match error.kind() {
            io::ErrorKind::AlreadyExists => Ok(()),
            _ => Err(error),
        })
        .map_err(|err| Error::Io { path: temp_dir.to_path_buf(), source: err })?;

    let s3_path = format!("uploaded-profile-image/{}/{}", user_type_str, id);

    Ok((temp_dir.join(id.to_string()), s3_path))
}
