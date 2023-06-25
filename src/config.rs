// Copyright 2023. The resback authors all rights reserved.

use std::io;

use jsonwebtoken::DecodingKey;

use crate::{
    env::get_env_or_panic,
    oauth::{OAuthConfig, OAuthProvider},
};

#[derive(Clone)]
pub struct Config {
    pub address: String,
    pub port: u16,
    pub database_url: String,
    pub front_url: String,

    pub google_oauth: OAuthConfig,
    pub kakao_oauth: OAuthConfig,
    pub naver_oauth: OAuthConfig,

    pub private_key: RSAKey,
    pub public_key: RSAKey,

    pub access_token_max_age: i64,
    pub refresh_token_max_age: i64,
}

#[derive(Clone)]
pub struct RSAKey {
    path: std::path::PathBuf,
    key: String,
    decoding_key: DecodingKey,
}

impl RSAKey {
    fn from_file(path: &std::path::PathBuf) -> io::Result<Self> {
        match std::fs::read_to_string(path) {
            Ok(key) => Ok(Self {
                path: path.to_path_buf(),
                key: key.clone(),
                decoding_key: DecodingKey::from_rsa_pem(key.as_bytes()).unwrap(),
            }),
            Err(err) => Err(err),
        }
    }

    pub fn as_bytes(self: &Self) -> &[u8] {
        self.key.as_bytes()
    }

    pub fn key(self: &Self) -> &str {
        &self.key
    }

    pub fn decoding_key(self: &Self) -> &DecodingKey {
        &self.decoding_key
    }
}

impl std::fmt::Display for RSAKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.key)
    }
}

impl Config {
    pub fn new() -> Self {
        let port: u16 = get_env_or_panic("PORT").parse().unwrap();

        Self {
            address: format!("0.0.0.0:{}", port),
            port,
            database_url: get_env_or_panic("MYSQL_DATABASE_URL"),
            front_url: env!("CARGO_PKG_HOMEPAGE").to_string(),

            google_oauth: OAuthConfig::init(OAuthProvider::Google),
            kakao_oauth: OAuthConfig::init(OAuthProvider::Kakao),
            naver_oauth: OAuthConfig::init(OAuthProvider::Naver),

            private_key: RSAKey::from_file(
                &std::path::PathBuf::from(get_env_or_panic("RSA_PRIVATE_PEM_FILE_PATH"))
                    .to_path_buf(),
            )
            .expect("Cannot open the private key file"),
            public_key: RSAKey::from_file(
                &std::path::PathBuf::from(get_env_or_panic("RSA_PUBLIC_PEM_FILE_PATH"))
                    .to_path_buf(),
            )
            .expect("Cannot open the public key file"),

            access_token_max_age: get_env_or_panic("ACCESS_TOKEN_MAX_AGE").parse::<i64>().unwrap(),
            refresh_token_max_age: get_env_or_panic("REFRESH_TOKEN_MAX_AGE")
                .parse::<i64>()
                .unwrap(),
        }
    }
}
