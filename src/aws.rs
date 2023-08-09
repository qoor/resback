// Copyright 2023. The resback authors all rights reserved.

use std::path::Path;

use aws_sdk_s3::primitives::ByteStream;

use crate::{error::Error, get_env_or_panic, Result};

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
            Error::FileToStreamFail { path: file_path.to_path_buf(), source: Box::new(err) }
        })?;

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(target_path)
            .body(body)
            .send()
            .await
            .map_err(|err| Error::UploadFail {
                path: file_path.to_path_buf(),
                source: Box::new(err),
            })?;

        Ok(format!("https://{}.s3.{}.amazonaws.com/{}", self.bucket, self.region, target_path))
    }
}

pub struct SesClient {
    client: aws_sdk_sesv2::Client,
}

impl SesClient {
    pub async fn from_env() -> Self {
        let aws_config = aws_config::load_from_env().await;

        Self { client: aws_sdk_sesv2::Client::new(&aws_config) }
    }

    pub async fn send_mail(
        &self,
        from: &str,
        to: &str,
        subject: &str,
        message: &str,
    ) -> Result<()> {
        let dest = aws_sdk_sesv2::types::Destination::builder().to_addresses(to).build();
        let subject =
            aws_sdk_sesv2::types::Content::builder().data(subject).charset("UTF-8").build();
        let body = aws_sdk_sesv2::types::Content::builder().data(message).charset("UTF-8").build();
        let body = aws_sdk_sesv2::types::Body::builder().text(body).build();

        let message = aws_sdk_sesv2::types::Message::builder().subject(subject).body(body).build();
        let content = aws_sdk_sesv2::types::EmailContent::builder().simple(message).build();

        self.client
            .send_email()
            .from_email_address(from)
            .destination(dest)
            .content(content)
            .send()
            .await
            .map_err(|err| Error::SendMailFail(Box::new(err)))?;

        Ok(())
    }
}
