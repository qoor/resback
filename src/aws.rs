// Copyright 2023. The resback authors all rights reserved.

use std::path::Path;

use aws_sdk_s3::primitives::ByteStream;
use axum::http::StatusCode;

use crate::{error::ErrorResponse, get_env_or_panic, Result};

pub struct S3Client {
    client: aws_sdk_s3::Client,
    region: String,
    bucket: String,
}

impl S3Client {
    pub async fn from_env() -> Self {
        let aws_config = aws_config::load_from_env().await;

        Self {
            client: aws_sdk_s3::Client::new(&aws_config),
            region: aws_config.region().unwrap().to_string(),
            bucket: get_env_or_panic("AWS_S3_BUCKET"),
        }
    }

    pub async fn push_file(&self, file_path: &Path, target_path: &str) -> Result<String> {
        let body = ByteStream::from_path(&file_path).await.map_err(|err| {
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

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(target_path)
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

        Ok(format!("https://{}.s3.{}.amazonaws.com/{}", self.bucket, self.region, target_path))
    }
}
