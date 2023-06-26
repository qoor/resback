// Copyright 2023. The resback authors all rights reserved.

use crate::oauth::OAuthProvider;

pub mod account;

#[derive(Debug)]
pub struct OAuthUserData {
    provider: OAuthProvider,
    id: String,
}

impl OAuthUserData {
    pub fn new(provider: OAuthProvider, id: &str) -> Self {
        Self { provider, id: id.to_string() }
    }
    pub fn provider(&self) -> OAuthProvider {
        self.provider
    }
    pub fn id(&self) -> &str {
        &self.id
    }
}
