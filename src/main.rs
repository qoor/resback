// Copyright 2023. The resback authors all rights reserved.

use axum::Server;
use dotenvy::dotenv;
use resback::get_env_or_panic;
use sqlx::mysql::MySqlPoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "resback=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables from '.env' file
    dotenv().ok();

    println!("Starting the server...");
    println!();

    // Init application config from dotenv
    let config = resback::Config::new();

    let pool = match MySqlPoolOptions::new().connect(&get_env_or_panic("DATABASE_URL")).await {
        Ok(pool) => {
            println!("Connection to the database is successful.");
            pool
        }
        Err(err) => {
            println!("Failed to connect to the database: {:?}", err);
            std::process::exit(1);
        }
    };

    let migration = sqlx::migrate!().run(&pool).await;
    if let Err(err) = migration {
        println!("Failed to migrate database: {:?}", err);
        std::process::exit(1);
    }

    let app = resback::app(&config, &pool).await;

    print_server_started(&config.address);
    Server::bind(&config.address.parse().unwrap()).serve(app.into_make_service()).await.unwrap();
}

fn print_server_started(address: &str) {
    println!();
    print!("{}", resback::about());
    println!("Server started successfully. (address: {})", address);
}
