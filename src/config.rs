// Copyright 2023. The resback authors all rights reserved.

use crate::{
    env::get_env_or_panic,
    oauth::{OAuthConfig, OAuthProvider},
};

#[derive(Debug, Clone)]
pub struct Config {
    pub address: String,
    pub port: u16,
    pub database_url: String,

    pub google_oauth: OAuthConfig,
    pub kakao_oauth: OAuthConfig,
    pub naver_oauth: OAuthConfig,
}

impl Config {
    pub fn new() -> Self {
        let port: u16 = get_env_or_panic("PORT").parse().unwrap();
        Self {
            address: format!("0.0.0.0:{}", port),
            port,
            database_url: get_env_or_panic("MYSQL_DATABASE_URL"),

            google_oauth: OAuthConfig::init(OAuthProvider::Google),
            kakao_oauth: OAuthConfig::init(OAuthProvider::Kakao),
            naver_oauth: OAuthConfig::init(OAuthProvider::Naver),
        }
    }
}
