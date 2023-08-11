// Copyright 2023. The resback authors all rights reserved.

use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use tracing::error;

use crate::user::{account::UserId, UserType};

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub(crate) type BoxDynError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("an error occurred with the database")]
    Database(#[from] sqlx::Error),
    #[error("an error occurred with the JWT token")]
    Token(jsonwebtoken::errors::Error),
    #[error("invalid token")]
    InvalidToken,
    #[error("token does not exist")]
    TokenNotExists,
    #[error("authentication required")]
    Unauthorized,
    #[error("verification failed")]
    Verification,
    #[error("the verification code has been expired")]
    VerificationExpired,
    #[error("{} user {id} not found", match user_type {
        UserType::SeniorUser => "senior",
        UserType::NormalUser => "normal"
    })]
    UserNotFound { user_type: UserType, id: UserId },
    #[error("invalid request data {data} (expected {expected:?} found {found:?})")]
    InvalidRequestData { data: String, expected: String, found: String },
    #[error("login failed")]
    Login,
    #[error("an error occurred while handling the password")]
    Hash(argon2::password_hash::Error),
    #[error("failed to upload file")]
    Upload { path: std::path::PathBuf, source: BoxDynError },
    #[error("an error occurred while processing the file to be uploaded")]
    FileToStream { path: std::path::PathBuf, source: BoxDynError },
    #[error("an error occurred while sending the email")]
    SendMail(BoxDynError),
    #[error("an error occurred while processing the file to be uploaded")]
    PersistFile { path: std::path::PathBuf, source: BoxDynError },
    #[error("an error occurred while processing the file to be uploaded")]
    Io { path: std::path::PathBuf, source: std::io::Error },
    #[error("unhandled exception")]
    Unhandled(BoxDynError),
}

impl Error {
    fn status(&self) -> StatusCode {
        match self {
            Error::Database(_err) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::InvalidToken => StatusCode::BAD_REQUEST,
            Error::Token(_err) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::TokenNotExists => StatusCode::BAD_REQUEST,
            Error::Unauthorized => StatusCode::UNAUTHORIZED,
            Error::Verification => StatusCode::UNAUTHORIZED,
            Error::VerificationExpired => StatusCode::GONE,
            Error::UserNotFound { user_type: _, id: _ } => StatusCode::NOT_FOUND,
            Error::InvalidRequestData { data: _field, expected: _, found: _ } => {
                StatusCode::BAD_REQUEST
            }
            Error::Login => StatusCode::UNAUTHORIZED,
            Error::Hash(_err) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Upload { path: _, source: _ } => StatusCode::INTERNAL_SERVER_ERROR,
            Error::FileToStream { path: _, source: _ } => StatusCode::INTERNAL_SERVER_ERROR,
            Error::SendMail(_err) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::PersistFile { path: _, source: _ } => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Io { path: _, source: _ } => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Unhandled(_err) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::Database(ref err) => error!("database error: {err}"),
            Error::Token(ref err) => error!("jsonwebtoken error: {err}"),
            Error::Hash(ref err) => error!("argon2 hash error: {err}"),
            Error::Upload { ref path, ref source } => {
                error!("failed to upload file {}: {source}", path.to_string_lossy())
            }
            Error::FileToStream { ref path, ref source } => {
                error!(
                    "failed to create byte stream from file {}: {source}",
                    path.to_string_lossy()
                )
            }
            Error::SendMail(ref err) => error!("failed to send the mail: {}", err),
            Error::PersistFile { ref path, ref source } => {
                error!("failed to persist the file {}: {source}", path.to_string_lossy())
            }
            Error::Io { ref path, ref source } => {
                error!("{} I/O error: {source}", path.to_string_lossy())
            }
            Error::Unhandled(ref err) => error!("unhandled error: {err}"),

            _ => (),
        }

        #[derive(Serialize)]
        struct ErrorResponse {
            message: String,
        }

        (self.status(), Json(ErrorResponse { message: self.to_string() })).into_response()
    }
}

impl From<jsonwebtoken::errors::Error> for Error {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        match value.kind() {
            jsonwebtoken::errors::ErrorKind::InvalidToken => Self::InvalidToken,
            _ => Self::Token(value),
        }
    }
}

impl From<argon2::password_hash::Error> for Error {
    fn from(value: argon2::password_hash::Error) -> Self {
        Self::Hash(value)
    }
}
