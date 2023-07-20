// Copyright 2023. The resback authors all rights reserved.

use std::{str::FromStr, time::Duration};

use chrono::{DateTime, Utc};
use oauth2::{
    basic::{
        BasicClient, BasicErrorResponse, BasicRevocationErrorResponse,
        BasicTokenIntrospectionResponse, BasicTokenType,
    },
    helpers, AccessToken, AuthUrl, Client, ClientId, ClientSecret, EmptyExtraTokenFields,
    ExtraTokenFields, RedirectUrl, RefreshToken, Scope, StandardRevocableToken,
    StandardTokenResponse, TokenResponse, TokenType, TokenUrl,
};
use serde::{Deserialize, Serialize};

use crate::env::get_env_or_panic;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, sqlx::Type)]
#[serde(rename_all = "snake_case")]
pub enum OAuthProvider {
    Google,
    Kakao,
    Naver,
}

impl FromStr for OAuthProvider {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "google" => Ok(OAuthProvider::Google),
            "kakao" => Ok(OAuthProvider::Kakao),
            "naver" => Ok(OAuthProvider::Naver),
            _ => Err(String::from("Invalid OAuthProvider string")),
        }
    }
}

impl std::fmt::Display for OAuthProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone)]
pub struct OAuthConfig {
    provider: OAuthProvider,
    client_id: String,
    auth_uri: String,
    token_uri: String,
    client_secret: String,
    redirect_uri: String,
    pub user_data_uri: String,
}

impl OAuthConfig {
    pub fn init(provider: OAuthProvider) -> Self {
        let env_prefix = provider.to_string().to_uppercase();
        let client_id_env = format!("{}_CLIENT_ID", env_prefix);
        let auth_uri_env = format!("{}_AUTH_URI", env_prefix);
        let token_uri_env = format!("{}_TOKEN_URI", env_prefix);
        let client_secret_env = format!("{}_CLIENT_SECRET", env_prefix);
        let redirect_uri_env = format!("{}_REDIRECT_URI", env_prefix);
        let user_data_uri_env = format!("{}_USER_DATA_URI", env_prefix);

        Self {
            provider,
            client_id: get_env_or_panic(&client_id_env).to_string(),
            auth_uri: get_env_or_panic(&auth_uri_env).to_string(),
            token_uri: get_env_or_panic(&token_uri_env).to_string(),
            client_secret: get_env_or_panic(&client_secret_env).to_string(),
            redirect_uri: get_env_or_panic(&redirect_uri_env).to_string(),
            user_data_uri: get_env_or_panic(&user_data_uri_env).to_string(),
        }
    }

    /// Returns a OAuth 2.0 client for a provider that conforms to the OAuth 2.0
    /// standard.
    pub fn to_client(&self) -> BasicClient {
        let client = BasicClient::new(
            ClientId::new(self.client_id.clone()),
            Some(ClientSecret::new(self.client_secret.clone())),
            AuthUrl::new(self.auth_uri.clone()).unwrap(),
            Some(TokenUrl::new(self.token_uri.clone()).unwrap()),
        )
        .set_redirect_uri(RedirectUrl::new(self.redirect_uri.clone()).unwrap());
        // For Kakao provider, the `client_secret` key must be present in the request
        // body.
        match self.provider {
            OAuthProvider::Kakao => client.set_auth_type(oauth2::AuthType::RequestBody),
            OAuthProvider::Naver => panic!("Naver OAuth 2.0 client must be a `NonStandardClient`"),
            _ => client,
        }
    }

    /// Returns a OAuth 2.0 client for an non-standard OAuth 2.0 provider. For
    /// more details, see [`NonStandardTokenresponse`].
    pub fn to_non_standard_client(&self) -> NonStandardClient {
        match self.provider {
            OAuthProvider::Naver => NonStandardClient::new(
                ClientId::new(self.client_id.clone()),
                Some(ClientSecret::new(self.client_secret.clone())),
                AuthUrl::new(self.auth_uri.clone()).unwrap(),
                Some(TokenUrl::new(self.token_uri.clone()).unwrap()),
            )
            .set_redirect_uri(RedirectUrl::new(self.redirect_uri.clone()).unwrap()),

            _ => panic!("OAuth 2.0 client other than Naver must be a `BasicClient`"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GoogleUser {
    pub id: String,
    pub email: String,
    pub verified_email: bool,
    pub name: String,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub picture: String,
    pub locale: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KakaoUser {
    pub id: u64,
    pub connected_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NaverUserResponse {
    #[serde(rename = "resultcode")]
    pub result_code: String,
    pub message: String,
    pub response: NaverUser,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NaverUser {
    pub id: String,
    // nickname: String,
    // name: String,
    // email: String,
    // gender: String,
    // age: String,
    // birthday: String,
    // profile_image: String,
    // birthyear: String,
    // mobile: String,
}

///
/// Custom Token Response type to replace the StandardTokenResponse provided by
/// oauth2-rs. This is required because Microsoft, Naver are not in
/// compliance with the RFC spec for oauth2.0
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NonStandardTokenResponse<EF, TT>
where
    EF: ExtraTokenFields,
    TT: TokenType,
{
    access_token: AccessToken,
    #[serde(bound = "TT: TokenType")]
    #[serde(deserialize_with = "helpers::deserialize_untagged_enum_case_insensitive")]
    token_type: TT,
    #[serde(skip_serializing_if = "Option::is_none")]
    expires_in: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    refresh_token: Option<RefreshToken>,
    #[serde(rename = "scope")]
    #[serde(deserialize_with = "helpers::deserialize_space_delimited_vec")]
    #[serde(serialize_with = "helpers::serialize_space_delimited_vec")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    scopes: Option<Vec<Scope>>,

    #[serde(bound = "EF: ExtraTokenFields")]
    #[serde(flatten)]
    extra_fields: EF,
}

#[allow(dead_code)]
impl<EF, TT> NonStandardTokenResponse<EF, TT>
where
    EF: ExtraTokenFields,
    TT: TokenType,
{
    pub fn new(access_token: AccessToken, token_type: TT, extra_fields: EF) -> Self {
        Self {
            access_token,
            token_type,
            expires_in: None,
            refresh_token: None,
            scopes: None,
            extra_fields,
        }
    }

    pub fn set_access_token(&mut self, access_token: AccessToken) {
        self.access_token = access_token;
    }

    pub fn set_token_type(&mut self, token_type: TT) {
        self.token_type = token_type;
    }

    pub fn set_expires_in(&mut self, expires_in: Option<&Duration>) {
        self.expires_in = expires_in.map(|exp| Duration::as_secs(exp).to_string());
    }

    pub fn set_refresh_token(&mut self, refresh_token: Option<RefreshToken>) {
        self.refresh_token = refresh_token;
    }

    pub fn set_scopes(&mut self, scopes: Option<Vec<Scope>>) {
        self.scopes = scopes;
    }

    pub fn extra_fields(&self) -> &EF {
        &self.extra_fields
    }

    pub fn set_extra_fields(&mut self, extra_fields: EF) {
        self.extra_fields = extra_fields;
    }
}

impl<EF, TT> TokenResponse<TT> for NonStandardTokenResponse<EF, TT>
where
    EF: ExtraTokenFields,
    TT: TokenType,
{
    ///
    /// The access token issued by the Azure, Naver authentication server
    fn access_token(&self) -> &AccessToken {
        &self.access_token
    }
    fn token_type(&self) -> &TT {
        &self.token_type
    }
    fn expires_in(&self) -> Option<Duration> {
        self.expires_in.as_ref().map(|exp| {
            let expires_in_number: u64 = exp.parse::<u64>().unwrap();

            Duration::from_secs(expires_in_number)
        })
    }
    fn refresh_token(&self) -> Option<&RefreshToken> {
        self.refresh_token.as_ref()
    }
    fn scopes(&self) -> Option<&Vec<Scope>> {
        self.scopes.as_ref()
    }
}

impl<EF, TT> From<StandardTokenResponse<EF, TT>> for NonStandardTokenResponse<EF, TT>
where
    EF: ExtraTokenFields + Clone,
    TT: TokenType,
{
    fn from(st: StandardTokenResponse<EF, TT>) -> Self {
        let expire_time_string = st.expires_in().map(|exp| Duration::as_secs(&exp).to_string());
        let extra_fields: EF = st.extra_fields().clone();
        Self {
            access_token: st.access_token().clone(),
            token_type: st.token_type().clone(),
            expires_in: expire_time_string,
            refresh_token: st.refresh_token().cloned(),
            scopes: st.scopes().cloned(),
            extra_fields,
        }
    }
}

pub type BasicNonStandardTokenResponse =
    NonStandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>;

pub type NonStandardClient = Client<
    BasicErrorResponse,
    BasicNonStandardTokenResponse,
    BasicTokenType,
    BasicTokenIntrospectionResponse,
    StandardRevocableToken,
    BasicRevocationErrorResponse,
>;
