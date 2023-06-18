use crate::get_env_or_panic;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    /*pub access_token: jwt::Token,
    pub refresh_token: jwt::Token,*/
}

impl Config {
    pub fn new() -> Self {
        Self { database_url: get_env_or_panic("MYSQL_DATABASE_URL") }
    }
}
