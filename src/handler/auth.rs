// Copyright 2023. The resback authors all rights reserved.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use axum_typed_multipart::TypedMultipart;
use oauth2::{
    reqwest::async_http_client, AuthorizationCode, ErrorResponse, RevocableToken,
    TokenIntrospectionResponse, TokenResponse, TokenType,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};

use crate::{
    error,
    jwt::{generate_jwt_token, get_user_info_from_token, verify_token},
    oauth::OAuthProvider,
    schema::{NormalLoginSchema, SeniorLoginSchema},
    user::account::{SeniorUser, UserId},
    AppState,
};
use crate::{
    jwt::{ACCESS_TOKEN_COOKIE, REFRESH_TOKEN_COOKIE},
    user::{
        account::{NormalUser, User},
        OAuthUserData, UserType,
    },
};

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct AuthRequest {
    code: String,
    state: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct GoogleUser {
    id: String,
    email: String,
    verified_email: bool,
    name: String,
    given_name: Option<String>,
    family_name: Option<String>,
    picture: String,
    locale: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct KakaoUser {
    id: u64,
    connected_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
struct NaverUserResponse {
    #[serde(rename = "resultcode")]
    result_code: String,
    message: String,
    response: NaverUser,
}

#[derive(Debug, Serialize, Deserialize)]
struct NaverUser {
    id: String,
    // nickname: String,
    // name: String,
    // email: String,
    // gender: String,
    // age: String,
    // birthday: String,
    // profile_image: String,
    // birthyear: String,
    // mobile: String,
}

pub async fn auth_provider(
    cookie_jar: CookieJar,
    Path(provider): Path<OAuthProvider>,
    State(data): State<Arc<AppState>>,
    TypedMultipart(login_data): TypedMultipart<NormalLoginSchema>,
) -> impl IntoResponse {
    let oauth_id: String;

    match provider {
        OAuthProvider::Google => {
            let google_user: GoogleUser = get_oauth_user_data(
                &data.google_oauth,
                &data.config.google_oauth.user_data_uri,
                &login_data.code,
            )
            .await;

            oauth_id = google_user.id.to_string();
        }
        OAuthProvider::Kakao => {
            let kakao_user: KakaoUser = get_oauth_user_data(
                &data.kakao_oauth,
                &data.config.kakao_oauth.user_data_uri,
                &login_data.code,
            )
            .await;
            oauth_id = kakao_user.id.to_string();
        }
        OAuthProvider::Naver => {
            let naver_user_response: NaverUserResponse = get_oauth_user_data(
                &data.naver_oauth,
                &data.config.naver_oauth.user_data_uri,
                &login_data.code,
            )
            .await;
            oauth_id = naver_user_response.response.id;
        }
    }

    let oauth_user = OAuthUserData::new(provider, &oauth_id);
    let user = match NormalUser::from_oauth_user(&oauth_user, &data.database).await {
        Ok(user) => user,
        Err(_) => {
            let user_id = NormalUser::register(&oauth_user, &data.database).await?;
            NormalUser::from_id(user_id, &data.database).await?
        }
    };

    add_token_pair_to_cookie_jar(&user, UserType::NormalUser, cookie_jar, &data).await
}

pub async fn auth_senior(
    cookie_jar: CookieJar,
    State(data): State<Arc<AppState>>,
    TypedMultipart(login_data): TypedMultipart<SeniorLoginSchema>,
) -> crate::Result<impl IntoResponse> {
    let user = SeniorUser::login(&login_data.email, &login_data.password, &data.database).await?;

    add_token_pair_to_cookie_jar(&user, UserType::SeniorUser, cookie_jar, &data).await
}

pub async fn auth_refresh(
    cookie_jar: CookieJar,
    State(data): State<Arc<AppState>>,
) -> crate::Result<impl IntoResponse> {
    let refresh_token = cookie_jar.get(REFRESH_TOKEN_COOKIE).map(|token| token.value().to_string());

    let (user_type, user_id) = get_user_info_from_token(refresh_token.as_deref(), &data).await?;
    let refresh_token = refresh_token.unwrap();

    let user_token = match user_type {
        UserType::NormalUser => {
            let user = NormalUser::from_id(user_id, &data.database).await?;
            user.refresh_token().map(str::to_string)
        }
        UserType::SeniorUser => {
            let user = SeniorUser::from_id(user_id, &data.database).await?;
            user.refresh_token().map(str::to_string)
        }
    };

    let user_token = user_token.ok_or((
        StatusCode::UNAUTHORIZED,
        error::ErrorResponse { status: "fail", message: "You are not logged in".to_string() },
    ))?;

    if refresh_token != user_token {
        return Err((
            StatusCode::UNAUTHORIZED,
            error::ErrorResponse {
                status: "fail",
                message: "Authorization data and user data do not match".to_string(),
            },
        ));
    }

    add_access_token_to_cookie_jar(user_id, user_type, cookie_jar, &data).await
}

pub async fn logout_user(
    cookie_jar: CookieJar,
    State(data): State<Arc<AppState>>,
) -> crate::Result<impl IntoResponse> {
    let access_token = cookie_jar.get(ACCESS_TOKEN_COOKIE).ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        (crate::error::ErrorResponse {
            status: "error",
            message: "Failed to get login information".to_string(),
        }),
    ))?;
    let refresh_token = cookie_jar.get(REFRESH_TOKEN_COOKIE).ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        (crate::error::ErrorResponse {
            status: "error",
            message: "Failed to get login information".to_string(),
        }),
    ))?;

    let claims = verify_token(data.config.public_key.decoding_key(), &access_token.to_string())
        .map(|token_data| token_data.claims)
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                crate::error::ErrorResponse {
                    status: "fail",
                    message: "Failed to verify user".to_string(),
                },
            )
        })?;
    Ok((
        cookie_jar.clone().remove(access_token.clone()).remove(refresh_token.clone()),
        Json(serde_json::json!({ "uid": claims.sub() })),
    ))
}

async fn get_oauth_user_data<U, TE, TR, TT, TIR, RT, TRE>(
    oauth_client: &oauth2::Client<TE, TR, TT, TIR, RT, TRE>,
    user_data_url: &str,
    authorization_code: &str,
) -> U
where
    U: DeserializeOwned,
    TE: ErrorResponse + 'static,
    TR: TokenResponse<TT>,
    TT: TokenType,
    TIR: TokenIntrospectionResponse<TT>,
    RT: RevocableToken,
    TRE: ErrorResponse + 'static,
{
    // Get an authorization token
    let token = oauth_client
        .exchange_code(AuthorizationCode::new(authorization_code.to_string()))
        .request_async(async_http_client)
        .await
        .unwrap();

    // Fetch user data from `user_data_url`
    let request_client = reqwest::Client::new();
    let user_data = request_client
        .get(user_data_url)
        .bearer_auth(token.access_token().secret())
        .send()
        .await
        .unwrap()
        .json::<U>()
        .await
        .unwrap();

    user_data
}

async fn add_access_token_to_cookie_jar(
    user_id: UserId,
    user_type: UserType,
    cookie_jar: CookieJar,
    data: &AppState,
) -> crate::Result<(CookieJar, Json<serde_json::Value>)> {
    let access_token = generate_jwt_token(
        data.config.private_key.encoding_key(),
        chrono::Duration::seconds(data.config.access_token_max_age),
        user_type,
        user_id,
    )?;

    Ok((
        cookie_jar.add(
            Cookie::build(ACCESS_TOKEN_COOKIE, access_token.encoded_token().to_string())
                .path("/")
                .http_only(true)
                .max_age(time::Duration::seconds(access_token.claims().expires_in()))
                .finish(),
        ),
        Json(serde_json::json!({ "id": user_id })),
    ))
}

async fn add_token_pair_to_cookie_jar<U>(
    user: &U,
    user_type: UserType,
    cookie_jar: CookieJar,
    data: &AppState,
) -> crate::Result<impl IntoResponse>
where
    U: User,
{
    let (cookie_jar, _response) =
        add_access_token_to_cookie_jar(user.id(), user_type, cookie_jar, data).await?;

    let refresh_token = generate_jwt_token(
        data.config.private_key.encoding_key(),
        chrono::Duration::seconds(data.config.refresh_token_max_age),
        user_type,
        user.id(),
    )?;

    user.update_refresh_token(refresh_token.encoded_token(), &data.database).await?;

    Ok((
        cookie_jar.add(
            Cookie::build(REFRESH_TOKEN_COOKIE, refresh_token.encoded_token().to_string())
                .path("/")
                .http_only(true)
                .max_age(time::Duration::seconds(refresh_token.claims().expires_in()))
                .finish(),
        ),
        Json(serde_json::json!({ "id": user.id() })),
    ))
}
