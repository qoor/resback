// Copyright 2023. The resback authors all rights reserved.

use std::sync::Arc;

use axum::{
    extract::State,
    headers::{authorization::Bearer, Authorization},
    http::Request,
    middleware::Next,
    response::IntoResponse,
    RequestPartsExt, TypedHeader,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey};
use serde::{Deserialize, Serialize};

use crate::{
    error::Error,
    user::{
        account::{NormalUser, SeniorUser, User, UserId},
        UserType,
    },
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
    pub fn expires_in(&self) -> i64 {
        self.exp - self.iat
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    claims: Claims,
    encoded_token: String,
    user_id: UserId,
    user_type: UserType,
}

impl Token {
    pub fn new(
        private_key: &EncodingKey,
        expires_in: Duration,
        user_type: UserType,
        user_id: UserId,
    ) -> Result<Token> {
        let claims = Claims {
            iss: "https://respec.team/api".to_string(),
            iat: Utc::now().timestamp(),
            exp: (Utc::now() + expires_in).timestamp(),
            sub: user_id.to_string(),
            nonce: user_type.to_string(),
        };

        Ok(jsonwebtoken::encode(
            &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256),
            &claims,
            private_key,
        )
        .map(|token| Token { claims, encoded_token: token, user_id, user_type })?)
    }

    pub fn from_encoded_token(
        encoded_token: Option<&str>,
        public_key: &DecodingKey,
    ) -> Result<Self> {
        let encoded_token =
            encoded_token.ok_or(Error::TokenNotExists).and_then(|encoded_token| {
                if encoded_token.is_empty() {
                    return Err(Error::InvalidToken);
                }

                Ok(encoded_token.to_string())
            })?;

        let claims = jsonwebtoken::decode::<Claims>(
            &encoded_token,
            public_key,
            &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256),
        )
        .map(|token| token.claims)?;

        let user_id =
            claims.sub.parse::<UserId>().map_err(|err| Error::Unhandled(Box::new(err)))?;
        let user_type: UserType = claims.nonce.parse::<UserType>().map_err(Error::Unhandled)?;

        Ok(Token { claims, encoded_token, user_id, user_type })
    }

    pub fn claims(&self) -> &Claims {
        &self.claims
    }

    pub fn encoded_token(&self) -> &str {
        &self.encoded_token
    }

    pub fn user_id(&self) -> UserId {
        self.user_id
    }

    pub fn user_type(&self) -> UserType {
        self.user_type
    }
}

pub async fn authorize_user<B>(
    cookies: CookieJar,
    State(data): State<Arc<AppState>>,
    req: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse> {
    let (mut parts, body) = req.into_parts();

    // Find the access token in the cookies
    //
    // If the access token does not exists as cookie, try to find it in the
    // Authorization header in HTTP headers
    let access_token = match cookies.get(ACCESS_TOKEN_COOKIE) {
        Some(access_token) => Some(access_token.value().to_string()),
        None => parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .ok()
            .map(|auth_value| auth_value.token().to_string()),
    };

    let (user_id, user_type) =
        Token::from_encoded_token(access_token.as_deref(), data.config.public_key.decoding_key())
            .map(|token| (token.user_id(), token.user_type()))?;

    let mut req = Request::from_parts(parts, body);

    // Include the account data to extensions
    match user_type {
        UserType::NormalUser => {
            req.extensions_mut().insert(NormalUser::from_id(user_id, &data.database).await?);
        }
        UserType::SeniorUser => {
            req.extensions_mut().insert(SeniorUser::from_id(user_id, &data.database).await?);
        }
    };

    // Execute the next middleware
    Ok(next.run(req).await)
}

pub async fn logout_user(
    cookie_jar: CookieJar,
    public_key: &DecodingKey,
) -> Result<(UserType, UserId, CookieJar)> {
    let access_token = cookie_jar.get(ACCESS_TOKEN_COOKIE).ok_or(Error::TokenNotExists)?;
    let _refresh_token = cookie_jar.get(REFRESH_TOKEN_COOKIE).ok_or(Error::TokenNotExists)?;

    let (user_type, id) = Token::from_encoded_token(Some(access_token.value()), public_key)
        .map(|token| (token.user_type(), token.user_id()))?;

    let access_token = Cookie::build(ACCESS_TOKEN_COOKIE, "").path("/").finish();
    let refresh_token = Cookie::build(REFRESH_TOKEN_COOKIE, "").path("/").finish();

    Ok((user_type, id, cookie_jar.remove(access_token).remove(refresh_token)))
}
