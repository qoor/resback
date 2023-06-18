use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};

use crate::get_env_or_panic;

#[derive(Debug, Clone)]
pub struct OAuthConfig {
    client_id: String,
    auth_uri: String,
    token_uri: String,
    client_secret: String,
    redirect_uri: String,
    pub user_data_uri: String,
}

impl OAuthConfig {
    fn init(env_prefix: &str) -> Self {
        let client_id_env = format!("{}_CLIENT_ID", env_prefix);
        let auth_uri_env = format!("{}_AUTH_URI", env_prefix);
        let token_uri_env = format!("{}_TOKEN_URI", env_prefix);
        let client_secret_env = format!("{}_CLIENT_SECRET", env_prefix);
        let redirect_uri_env = format!("{}_REDIRECT_URI", env_prefix);
        let user_data_uri_env = format!("{}_USER_DATA_URI", env_prefix);

        Self {
            client_id: get_env_or_panic(&client_id_env).to_string(),
            auth_uri: get_env_or_panic(&auth_uri_env).to_string(),
            token_uri: get_env_or_panic(&token_uri_env).to_string(),
            client_secret: get_env_or_panic(&client_secret_env).to_string(),
            redirect_uri: get_env_or_panic(&redirect_uri_env).to_string(),
            user_data_uri: get_env_or_panic(&user_data_uri_env).to_string(),
        }
    }

    pub fn to_client(self: &Self) -> BasicClient {
        BasicClient::new(
            ClientId::new(self.client_id.clone()),
            Some(ClientSecret::new(self.client_secret.clone())),
            AuthUrl::new(self.auth_uri.clone()).unwrap(),
            Some(TokenUrl::new(self.token_uri.clone()).unwrap()),
        )
        .set_redirect_uri(RedirectUrl::new(self.redirect_uri.clone()).unwrap())
    }
}

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

            google_oauth: OAuthConfig::init("GOOGLE"),
            kakao_oauth: OAuthConfig::init("KAKAO"),
            naver_oauth: OAuthConfig::init("NAVER"),
        }
    }
}
