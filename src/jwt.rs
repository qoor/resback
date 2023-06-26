// Copyright 2023. The resback authors all rights reserved.

use std::sync::Arc;

use axum::{
    extract::State,
    headers::{authorization::Bearer, Authorization},
    http::{request, Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
    Json, RequestPartsExt, TypedHeader,
};
use axum_extra::extract::CookieJar;
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey};
use serde::{Deserialize, Serialize};

use crate::{
    error::ErrorResponse,
    user::account::{NormalUser, User, UserId, UserType},
    AppState, Result,
};

pub const ACCESS_TOKEN_COOKIE: &str = "access_token";
pub const REFRESH_TOKEN_COOKIE: &str = "refresh_token";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Issuer of the JWT
    iss: String,
    /// Time at which the JWT was issued; can be used to determine age of the
    /// JWT
    iat: i64,
    /// Time after which the JWT expires
    exp: i64,
    /// Subject of the JWT (the user)
    sub: String,
    /// Value used to associate a Client session with an ID Token (MAY also be
    /// used for nonce values in other applications of JWTs)
    /// It is used to know the account type ([`NormalUser`] as "normal" and
    /// [`SeniorUser`] as "senior")
    nonce: String,
}

impl Claims {
    pub fn sub(&self) -> &str {
        &self.sub
    }

    pub fn expires_in(&self) -> i64 {
        self.exp - self.iat
    }

    pub fn nonce(&self) -> &str {
        &self.nonce
    }
}

#[derive(Debug, Clone)]
pub struct TokenData {
    claims: Claims,
    encoded_token: String,
}

impl TokenData {
    pub fn claims(&self) -> &Claims {
        &self.claims
    }

    pub fn encoded_token(&self) -> &str {
        &self.encoded_token
    }
}

pub fn generate_jwt_token(
    private_key: &EncodingKey,
    expires_in: Duration,
    user_type: UserType,
    user_id: UserId,
) -> axum::response::Result<TokenData, Json<ErrorResponse>> {
    let claims = Claims {
        iss: "https://respec.team/api".to_string(),
        iat: Utc::now().timestamp(),
        exp: (Utc::now() + expires_in).timestamp(),
        sub: user_id.to_string(),
        nonce: user_type.to_string(),
    };

    jsonwebtoken::encode(
        &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256),
        &claims,
        &private_key,
    )
    .map(|token| Ok(TokenData { claims, encoded_token: token }))
    .map_err(|_| {
        Json(ErrorResponse { status: "fail", message: "Failed to generate token".to_string() })
    })?
}

pub async fn authorize_normal_user<B>(
    cookies: CookieJar,
    State(data): State<Arc<AppState>>,
    req: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse> {
    let (mut parts, body) = req.into_parts();
    let (user_type, user_id) = authorize_user(&mut parts, &cookies, &data).await?;

    let mut req = Request::from_parts(parts, body);

    // Include the account data to extensions
    match user_type {
        UserType::NormalUser => {
            req.extensions_mut().insert(NormalUser::from_id(user_id, &data.database).await?)
        }
        UserType::SeniorUser => unimplemented!(),
    };

    // Execute the next middleware
    Ok(next.run(req).await)
}

async fn authorize_user(
    parts: &mut request::Parts,
    cookies: &CookieJar,
    data: &Arc<AppState>,
) -> Result<(UserType, UserId)> {
    // Find the access token in the cookies
    //
    // If the access token does not exists as cookie, try to find it in the
    // Authorization header in HTTP headers
    let access_token = match cookies.get(ACCESS_TOKEN_COOKIE) {
        Some(access_token) => Some(access_token.to_string()),
        None => parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .ok()
            .map(|auth_value| auth_value.token().to_string()),
    };

    let access_token = access_token.ok_or_else(|| {
        let error_response = ErrorResponse {
            status: "fail",
            message: "You are not logged in. Please provide token".to_string(),
        };
        (StatusCode::UNAUTHORIZED, Json(error_response))
    })?;

    // Check if the access token has expired or invalid
    let claims = verify_token(data.config.public_key.decoding_key(), &access_token)
        .map_err(|_| {
            let error_response = ErrorResponse {
                status: "fail",
                message: "Your token is invalid or session has expired".to_string(),
            };
            (StatusCode::UNAUTHORIZED, Json(error_response))
        })
        .map(|token| token.claims)?;

    let user_type = claims.nonce.parse::<UserType>().map_err(|_| {
        let error_response =
            ErrorResponse { status: "fail", message: "Unknown user type".to_string() };
        (StatusCode::UNAUTHORIZED, Json(error_response))
    })?;

    Ok((user_type, claims.sub().parse::<UserId>().unwrap()))
}

/// Returns the full JWT token data if valid, otherwise returns an error
///
/// According to jsonwebtoken library, decoding RSA pem key is very
/// expensive. So it takes an already decoded key.
pub fn verify_token(
    decoding_key: &DecodingKey,
    token: &str,
) -> jsonwebtoken::errors::Result<jsonwebtoken::TokenData<Claims>> {
    jsonwebtoken::decode::<Claims>(
        token,
        &decoding_key,
        &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256),
    )
}
