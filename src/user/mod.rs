// Copyright 2023. The resback authors all rights reserved.

use std::str::FromStr;

use crate::oauth::OAuthProvider;

pub mod account;
pub mod picture;

#[derive(Debug, Clone, Copy)]
pub enum UserType {
    NormalUser,
    SeniorUser,
}

impl FromStr for UserType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "NormalUser" => Ok(Self::NormalUser),
            "SeniorUser" => Ok(Self::SeniorUser),
            _ => Err("Invalid user type string".to_string()),
        }
    }
}

impl std::fmt::Display for UserType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

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
