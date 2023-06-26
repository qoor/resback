// Copyright 2023. The resback authors all rights reserved.

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use oauth2::{
    reqwest::async_http_client, AuthorizationCode, CsrfToken, ErrorResponse, RevocableToken, Scope,
    TokenIntrospectionResponse, TokenResponse, TokenType,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};

use crate::user::{
    account::{NormalUser, User, UserType},
    OAuthUserData,
};
use crate::{jwt::generate_jwt_token, oauth::OAuthProvider, AppState};

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
    given_name: String,
    family_name: String,
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

pub async fn auth_google_handler(State(data): State<Arc<AppState>>) -> impl IntoResponse {
    let (auth_url, _csrf_token) = data
        .google_oauth
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("https://www.googleapis.com/auth/userinfo.email".to_string()))
        .add_scope(Scope::new("https://www.googleapis.com/auth/userinfo.profile".to_string()))
        .url();

    Redirect::to(auth_url.as_ref())
}

pub async fn auth_kakao_handler(State(data): State<Arc<AppState>>) -> impl IntoResponse {
    let (auth_url, _csrf_token) = data.kakao_oauth.authorize_url(CsrfToken::new_random).url();

    Redirect::to(auth_url.as_ref())
}

pub async fn auth_naver_handler(State(data): State<Arc<AppState>>) -> impl IntoResponse {
    let (auth_url, _csrf_token) = data.naver_oauth.authorize_url(CsrfToken::new_random).url();

    Redirect::to(auth_url.as_ref())
}

pub async fn auth_provider_authorized_handler(
    cookie_jar: CookieJar,
    Path(provider): Path<OAuthProvider>,
    Query(query): Query<AuthRequest>,
    State(data): State<Arc<AppState>>,
) -> axum::response::Result<(CookieJar, impl IntoResponse)> {
    let oauth_id: String;

    match provider {
        OAuthProvider::Google => {
            let google_user: GoogleUser = get_oauth_user_data(
                &data.google_oauth,
                &data.config.google_oauth.user_data_uri,
                &query.code,
            )
            .await;

            oauth_id = google_user.id.to_string();
        }
        OAuthProvider::Kakao => {
            let kakao_user: KakaoUser = get_oauth_user_data(
                &data.kakao_oauth,
                &data.config.kakao_oauth.user_data_uri,
                &query.code,
            )
            .await;
            oauth_id = kakao_user.id.to_string();
        }
        OAuthProvider::Naver => {
            let naver_user_response: NaverUserResponse = get_oauth_user_data(
                &data.naver_oauth,
                &data.config.naver_oauth.user_data_uri,
                &query.code,
            )
            .await;
            oauth_id = naver_user_response.response.id;
        }
    }

    let oauth_user = OAuthUserData::new(provider, &oauth_id);
    let user = NormalUser::login(&oauth_user, &data.database).await;
    let user = match user {
        Ok(user) => user,
        Err(_) => {
            let user_id = NormalUser::register(&oauth_user, &data.database).await?;
            NormalUser::from_id(user_id, &data.database).await?
        }
    };

    let access_token = generate_jwt_token(
        data.config.private_key.encoding_key(),
        chrono::Duration::seconds(data.config.access_token_max_age),
        UserType::NormalUser,
        user.id(),
    )
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))?;
    let refresh_token = generate_jwt_token(
        data.config.private_key.encoding_key(),
        chrono::Duration::seconds(data.config.refresh_token_max_age),
        UserType::NormalUser,
        user.id(),
    )
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))?;

    user.update_refresh_token(refresh_token.encoded_token(), &data.database).await?;

    Ok((
        cookie_jar.add(
            Cookie::build("access_token", access_token.encoded_token().to_string())
                .path("/")
                .http_only(true)
                .max_age(time::Duration::seconds(access_token.claims().expires_in()))
                .finish(),
        ),
        StatusCode::OK,
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
