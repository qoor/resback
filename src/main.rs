// Copyright 2023. The resback authors all rights reserved.

mod config;
mod env;
mod handler;
mod jwt;
mod nickname;
mod oauth;
mod schema;
mod user;

use std::sync::Arc;

use axum::{routing::get, Router, Server};
use config::Config;
use dotenv::dotenv;
use oauth::NonStandardClient;
use sqlx::{mysql::MySqlPoolOptions, MySql};

pub struct AppState {
    database: sqlx::Pool<MySql>,
    config: Config,
    google_oauth: oauth2::basic::BasicClient,
    kakao_oauth: oauth2::basic::BasicClient,
    /// Naver OAuth 2.0 returns token response in a non-standard way.
    /// If you run OAuth client as `BasicClient`, we will get a parsing error.
    /// Bugs:
    /// * https://github.com/ramosbugs/oauth2-rs/issues/191
    naver_oauth: NonStandardClient,
}

#[tokio::main]
async fn main() {
    // Load environment variables from '.env' file
    dotenv().ok();

    println!("Starting the server...");
    println!();

    // Init application config from dotenv
    let config = Config::new();

    let pool = match MySqlPoolOptions::new().connect(&config.database_url).await {
        Ok(pool) => {
            println!("Connection to the database is successful.");
            pool
        }
        Err(err) => {
            println!("Failed to connect to the database: {:?}", err);
            std::process::exit(1);
        }
    };

    let app = Router::new()
        .route("/", get(handler::root_handler))
        // For Google OAuth 2.0
        .route("/auth/google", get(handler::auth::auth_google_handler))
        .route("/auth/google/authorized", get(handler::auth::auth_google_authorized_handler))
        // For Kakao OAuth 2.0
        .route("/auth/kakao", get(handler::auth::auth_kakao_handler))
        .route("/auth/kakao/authorized", get(handler::auth::auth_kakao_authorized_handler))
        // For Naver OAuth 2.0
        .route("/auth/naver", get(handler::auth::auth_naver_handler))
        .route("/auth/naver/authorized", get(handler::auth::auth_naver_authorized_handler))
        // Sharing application state
        .with_state(Arc::new(AppState {
            database: pool.clone(),
            config: config.clone(),
            google_oauth: config.google_oauth.to_client(),
            kakao_oauth: config.kakao_oauth.to_client(),
            naver_oauth: config.naver_oauth.to_non_standard_client(),
        }));

    print_server_started(&config.address);
    Server::bind(&config.address.parse().unwrap()).serve(app.into_make_service()).await.unwrap();
}

pub fn about() -> String {
    const NAME: &str = env!("CARGO_PKG_NAME");
    const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    let authors: Vec<&str> = env!("CARGO_PKG_AUTHORS").split(':').collect();
    format!(
        "{NAME} - {DESCRIPTION}
Version: {VERSION}
Authors: {:?}\n",
        authors
    )
}

fn print_server_started(address: &str) {
    println!();
    print!("{}", about());
    println!("Server started successfully. (address: {})", address);
}
