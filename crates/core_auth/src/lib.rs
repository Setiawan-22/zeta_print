use tracing::info;
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

const SECRET_KEY: &[u8] = b"zetaprint_super_secret_key_change_me_in_prod";

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub role: String,
    pub exp: usize,
}

pub async fn init() -> Result<(), anyhow::Error> {
    info!("Initializing core_auth module...");
    // Future expansion: Setup Active Directory / LDAP connection pools or static secret keys for JWT
    Ok(())
}

pub fn create_jwt(user_id: &str, username: &str, role: &str) -> Result<String, anyhow::Error> {
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs() as usize + (24 * 3600); // 24 hours validity

    let claims = Claims {
        sub: user_id.to_owned(),
        username: username.to_owned(),
        role: role.to_owned(),
        exp: expiration,
    };

    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(SECRET_KEY))?;
    Ok(token)
}

pub async fn validate_token(token: &str) -> Result<AuthUser, anyhow::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(SECRET_KEY),
        &Validation::default()
    ).map_err(|e| anyhow::anyhow!("Invalid token: {}", e))?;

    Ok(AuthUser {
        id: token_data.claims.sub,
        username: token_data.claims.username,
        role: token_data.claims.role,
    })
}

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: String,
    pub username: String,
    pub role: String,
}
