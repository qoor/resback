// Copyright 2023 The resback authors

use std::time::Duration;

use oauth2::{
    basic::{
        BasicErrorResponse, BasicRevocationErrorResponse, BasicTokenIntrospectionResponse,
        BasicTokenType,
    },
    helpers, AccessToken, Client, EmptyExtraTokenFields, ExtraTokenFields, RefreshToken, Scope,
    StandardRevocableToken, StandardTokenResponse, TokenResponse, TokenType,
};
use serde::{Deserialize, Serialize};

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
            refresh_token: st.refresh_token().map(|r| r.clone()),
            scopes: st.scopes().map(|s| s.clone()),
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
