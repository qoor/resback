// Copyright 2023. The resback authors all rights reserved.

use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

use crate::user::{account::UserId, UserType};

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Error,
    Fail,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub status: Status,
    pub message: String,
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub(crate) type BoxDynError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug)]
pub enum Error {
    Database(sqlx::Error),
    Token(jsonwebtoken::errors::Error),
    InvalidToken,
    TokenNotExists,
    Unauthorized,
    VerificationExpired,
    UserNotFound { user_type: UserType, id: UserId },
    InvalidRequestData,
    LoginFail,
    HashFail(argon2::password_hash::Error),
    UploadFail { path: std::path::PathBuf, source: BoxDynError },
    FileToStreamFail { path: std::path::PathBuf, source: BoxDynError },
    SendMailFail(BoxDynError),
    PersistFileFail(BoxDynError),
    Io { path: std::path::PathBuf, source: std::io::Error },
    UnhandledException(BoxDynError),
}

impl Error {
    fn to_response(&self) -> (StatusCode, ErrorResponse) {
        match self {
            Error::Database(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse { status: Status::Error, message: format!("database error: {err}") },
            ),
            Error::InvalidToken => (
                StatusCode::BAD_REQUEST,
                ErrorResponse { status: Status::Fail, message: "invalid token".to_string() },
            ),
            Error::Token(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    status: Status::Error,
                    message: format!("failed to handle token: {err}"),
                },
            ),
            Error::TokenNotExists => (
                StatusCode::BAD_REQUEST,
                ErrorResponse { status: Status::Fail, message: "token does not exist".to_string() },
            ),
            Error::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                ErrorResponse {
                    status: Status::Fail,
                    message: "token is invalid or expired".to_string(),
                },
            ),
            Error::VerificationExpired => (
                StatusCode::UNAUTHORIZED,
                ErrorResponse {
                    status: Status::Fail,
                    message: "verification code has been expired".to_string(),
                },
            ),
            Error::UserNotFound { user_type, id } => (
                StatusCode::NOT_FOUND,
                ErrorResponse {
                    status: Status::Fail,
                    message: format!("user not found (type: {user_type}, id: {id})"),
                },
            ),
            Error::InvalidRequestData => (
                StatusCode::BAD_REQUEST,
                ErrorResponse { status: Status::Fail, message: "invalid request data".to_string() },
            ),
            Error::LoginFail => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    status: Status::Fail,
                    message: "invalid email or password".to_string(),
                },
            ),
            Error::HashFail(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    status: Status::Error,
                    message: format!("failed to hash raw password: {err}"),
                },
            ),
            Error::UploadFail { path, source } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    status: Status::Error,
                    message: format!("failed to upload file {}: {source}", path.to_string_lossy(),),
                },
            ),
            Error::FileToStreamFail { path, source } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    status: Status::Error,
                    message: format!(
                        "failed to create byte stream from file {}: {source}",
                        path.to_string_lossy()
                    ),
                },
            ),
            Error::SendMailFail(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    status: Status::Error,
                    message: format!("failed to send email: {err}"),
                },
            ),
            Error::PersistFileFail(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    status: Status::Error,
                    message: format!("failed to persist temporary file: {err}"),
                },
            ),
            Error::Io { path, source } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    status: Status::Error,
                    message: format!("{} I/O failed: {source}", path.to_string_lossy()),
                },
            ),
            Error::UnhandledException(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    status: Status::Error,
                    message: format!("unhandled exception: {err}"),
                },
            ),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.to_response())
    }
}

impl std::error::Error for Error {}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        self.to_response().into_response()
    }
}

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value)
    }
}

impl From<argon2::password_hash::Error> for Error {
    fn from(value: argon2::password_hash::Error) -> Self {
        Self::HashFail(value)
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
