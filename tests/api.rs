use axum::{body::Body, http::Request};
use reqwest::StatusCode;
use resback::{app, Config};
use sqlx::{MySql, Pool};
use tower::ServiceExt;

#[sqlx::test]
async fn root(pool: Pool<MySql>) {
    let app = app(&Config::default(), &pool);

    let response =
        app.oneshot(Request::builder().uri("/").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();

    const NAME: &str = env!("CARGO_PKG_NAME");
    const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    let authors: Vec<&str> = env!("CARGO_PKG_AUTHORS").split(':').collect();
    const HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");

    let about = format!(
        "{NAME} - {DESCRIPTION}
{}

Version: {VERSION}
Authors: {:?}
\n",
        HOMEPAGE, authors
    )
    .into_bytes();

    assert_eq!(&body[..], &about[..]);
}
