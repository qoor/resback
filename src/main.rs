mod env;
mod handler;

use axum::{routing::get, Router, Server};
use dotenv::dotenv;
use env::get_env_or_panic;

#[tokio::main]
async fn main() {
    // Load environment variables from '.env' file
    dotenv().ok();

    let app = Router::new().route("/", get(handler::root_handler));

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
    print!("{}", about());
    println!("Server started successfully. (address: {})", address);
}
