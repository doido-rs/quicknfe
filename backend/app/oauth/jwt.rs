use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::mcp::config::McpConfig;

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessClaims {
    pub sub: String,
    pub jti: String,
    pub exp: usize,
    pub token_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshClaims {
    pub sub: String,
    pub jti: String,
    pub exp: usize,
    pub token_type: String,
}

pub fn issue_access_token(user_id: i64, jti: &str, ttl_secs: u64) -> jsonwebtoken::errors::Result<String> {
    let cfg = McpConfig::get();
    let exp = chrono::Utc::now().timestamp() as usize + ttl_secs as usize;
    let claims = AccessClaims {
        sub: user_id.to_string(),
        jti: jti.to_string(),
        exp,
        token_type: "access".into(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(cfg.oauth_signing_secret.as_bytes()),
    )
}

pub fn issue_refresh_token(user_id: i64, jti: &str, ttl_secs: u64) -> jsonwebtoken::errors::Result<String> {
    let cfg = McpConfig::get();
    let exp = chrono::Utc::now().timestamp() as usize + ttl_secs as usize;
    let claims = RefreshClaims {
        sub: user_id.to_string(),
        jti: jti.to_string(),
        exp,
        token_type: "refresh".into(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(cfg.oauth_signing_secret.as_bytes()),
    )
}

pub fn verify_access_token(token: &str) -> jsonwebtoken::errors::Result<AccessClaims> {
    let cfg = McpConfig::get();
    let data = decode::<AccessClaims>(
        token,
        &DecodingKey::from_secret(cfg.oauth_signing_secret.as_bytes()),
        &Validation::default(),
    )?;
    if data.claims.token_type != "access" {
        return Err(jsonwebtoken::errors::Error::from(
            jsonwebtoken::errors::ErrorKind::InvalidToken,
        ));
    }
    Ok(data.claims)
}

pub fn verify_refresh_token(token: &str) -> jsonwebtoken::errors::Result<RefreshClaims> {
    let cfg = McpConfig::get();
    let data = decode::<RefreshClaims>(
        token,
        &DecodingKey::from_secret(cfg.oauth_signing_secret.as_bytes()),
        &Validation::default(),
    )?;
    if data.claims.token_type != "refresh" {
        return Err(jsonwebtoken::errors::Error::from(
            jsonwebtoken::errors::ErrorKind::InvalidToken,
        ));
    }
    Ok(data.claims)
}
