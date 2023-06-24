use crate::oauth::OAuthProvider;

pub mod account;

#[derive(Debug)]
pub struct OAuthUserData {
    provider: OAuthProvider,
    id: String,
}
