// Copyright 2023 The resback authors

mod config;
mod env;
mod handler;
mod jwt;

use std::sync::Arc;

use axum::{routing::get, Router, Server};
use config::Config;
use dotenv::dotenv;
use env::get_env_or_panic;
use sqlx::{mysql::MySqlPoolOptions, MySql};

pub struct AppState {
    database: sqlx::Pool<MySql>,
    config: Config,
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
        // Sharing application state
        .with_state(Arc::new(AppState { database: pool.clone(), config }));

    let address = format!("0.0.0.0:{}", get_env_or_panic("PORT"));
    print_server_started(&address);
    Server::bind(&address.parse().unwrap()).serve(app.into_make_service()).await.unwrap();
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
