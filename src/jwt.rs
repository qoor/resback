use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Issuer of the JWT
    iss: String,
    /// Time at which the JWT was issued; can be used to determine age of the
    /// JWT
    iat: i64,
    /// Time after which the JWT expires
    exp: i64,
    /// Subject of the JWT (the user)
    sub: String,
}

pub fn generate_jwt_token(
    private_key: &[u8],
    expires_in: Duration,
    user_id: u32,
) -> jsonwebtoken::errors::Result<String> {
    let claims = Claims {
        iss: "https://respec.team/api".to_string(),
        iat: Utc::now().timestamp(),
        exp: (Utc::now() + expires_in).timestamp(),
        sub: user_id.to_string(),
    };

    jsonwebtoken::encode(
        &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256),
        &claims,
        &jsonwebtoken::EncodingKey::from_rsa_pem(private_key)?,
    )
}

pub fn is_valid_token(public_key: &[u8], token: &str) -> bool {
    match jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_rsa_pem(public_key).unwrap(),
        &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256),
    ) {
        Ok(_) => true,
        Err(_) => false,
    }
}
