use std::sync::Arc;

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
};
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthorizationCode, CsrfToken, Scope,
    TokenResponse,
};
use serde::Deserialize;

use crate::AppState;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct AuthRequest {
    code: String,
    state: String,
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

pub async fn auth_google_authorized_handler(
    Query(query): Query<AuthRequest>,
    State(data): State<Arc<AppState>>,
) -> String {
    get_user_data(&data.google_oauth, &data.config.google_oauth.user_data_uri, &query.code).await
}

async fn get_user_data(
    oauth_client: &BasicClient,
    user_data_url: &str,
    authorization_code: &str,
) -> String {
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
        .text()
        .await
        .unwrap();

    user_data
}
