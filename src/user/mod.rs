use crate::oauth::OAuthProvider;

mod account;

#[derive(Debug)]
pub struct OAuthUserData {
    provider: OAuthProvider,
    id: String,
}
