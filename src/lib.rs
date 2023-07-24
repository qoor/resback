// Copyright 2023. The resback authors all rights reserved.

pub mod config;
pub mod env;
mod error;
mod handler;
mod jwt;
mod nickname;
mod oauth;
mod schema;
mod user;

use sqlx::MySql;

use std::sync::Arc;

use axum::{
    middleware,
    routing::{delete, get, patch, post},
    Router,
};
use oauth::NonStandardClient;

pub use config::Config;
pub use env::get_env_or_panic;
pub use error::Result;

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

pub fn app(config: &Config, pool: &sqlx::Pool<MySql>) -> Router {
    let app_state = Arc::new(AppState {
        database: pool.clone(),
        config: config.clone(),
        google_oauth: config.google_oauth.to_client(),
        kakao_oauth: config.kakao_oauth.to_client(),
        naver_oauth: config.naver_oauth.to_non_standard_client(),
    });

    let auth_layer = middleware::from_fn_with_state(app_state.clone(), jwt::authorize_user);

    let root_routers = Router::new().route("/", get(handler::root));
    let auth_routers = Router::new()
        .route("/auth/:provider", post(handler::auth::auth_provider))
        .route("/auth/senior", post(handler::auth::auth_senior))
        .route("/auth/token", patch(handler::auth::auth_refresh).route_layer(auth_layer.clone()))
        .route("/auth/token", delete(handler::auth::logout_user).route_layer(auth_layer.clone()));
    let users_routers = Router::new()
        .route(
            "/users/senior",
            post(handler::users::register_senior_user).get(handler::users::get_seniors),
        )
        .route("/users/senior/:id", get(handler::users::get_senior_user_info))
        .route("/users/senior/:id", delete(handler::users::delete_senior_user))
        .route("/users/normal/:id", get(handler::users::get_normal_user_info))
        .route("/users/normal/:id", delete(handler::users::delete_normal_user));

    Router::new().merge(root_routers).merge(auth_routers).merge(users_routers).with_state(app_state)
}

pub fn about() -> String {
    const NAME: &str = env!("CARGO_PKG_NAME");
    const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    let authors: Vec<&str> = env!("CARGO_PKG_AUTHORS").split(':').collect();
    const HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");
    format!(
        "{NAME} - {DESCRIPTION}
{}

Version: {VERSION}
Authors: {:?}
\n",
        HOMEPAGE, authors
    )
}
