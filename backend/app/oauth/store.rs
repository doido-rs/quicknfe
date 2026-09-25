use std::sync::OnceLock;

use chrono::{Duration, Utc};
use doido::model::sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
};

use super::entities::{
    oauth_access_tokens, oauth_authorization_codes, oauth_clients, oauth_mcp_sessions,
};
use super::pkce::random_token;

pub async fn find_client(db: &DatabaseConnection, client_id: &str) -> doido::Result<Option<oauth_clients::Model>> {
    Ok(oauth_clients::Entity::find()
        .filter(oauth_clients::Column::ClientId.eq(client_id))
        .one(db)
        .await?)
}

pub fn redirect_uris(client: &oauth_clients::Model) -> Vec<String> {
    serde_json::from_str(&client.redirect_uris).unwrap_or_default()
}

pub fn redirect_uri_allowed(client: &oauth_clients::Model, redirect_uri: &str) -> bool {
    redirect_uris(client)
        .iter()
        .any(|u| u == redirect_uri)
}

pub async fn register_client(
    db: &DatabaseConnection,
    redirect_uris: Vec<String>,
) -> doido::Result<oauth_clients::Model> {
    let client_id = format!("qnfe_{}", random_token(16));
    let now = Utc::now();
    oauth_clients::ActiveModel {
        client_id: Set(client_id),
        client_secret: Set(None),
        redirect_uris: Set(serde_json::to_string(&redirect_uris)?),
        created_at: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await
    .map_err(Into::into)
}

pub async fn insert_auth_code(
    db: &DatabaseConnection,
    client_id: &str,
    user_id: i64,
    code_challenge: &str,
    redirect_uri: &str,
    session_cookie: &str,
) -> doido::Result<String> {
    let code = random_token(32);
    let expires_at = Utc::now() + Duration::minutes(10);
    oauth_authorization_codes::ActiveModel {
        code: Set(code.clone()),
        client_id: Set(client_id.to_string()),
        user_id: Set(user_id),
        code_challenge: Set(code_challenge.to_string()),
        redirect_uri: Set(redirect_uri.to_string()),
        session_cookie: Set(session_cookie.to_string()),
        expires_at: Set(expires_at),
        used: Set(false),
        ..Default::default()
    }
    .insert(db)
    .await?;
    Ok(code)
}

pub async fn consume_auth_code(
    db: &DatabaseConnection,
    code: &str,
    client_id: &str,
    redirect_uri: &str,
) -> doido::Result<Option<oauth_authorization_codes::Model>> {
    let row = oauth_authorization_codes::Entity::find()
        .filter(oauth_authorization_codes::Column::Code.eq(code))
        .filter(oauth_authorization_codes::Column::ClientId.eq(client_id))
        .filter(oauth_authorization_codes::Column::Used.eq(false))
        .one(db)
        .await?;
    let Some(row) = row else {
        return Ok(None);
    };
    if row.redirect_uri != redirect_uri {
        return Ok(None);
    }
    if row.expires_at < Utc::now() {
        return Ok(None);
    }
    let mut active: oauth_authorization_codes::ActiveModel = row.clone().into();
    active.used = Set(true);
    active.update(db).await?;
    Ok(Some(row))
}

pub async fn insert_access_token(
    db: &DatabaseConnection,
    jti: &str,
    user_id: i64,
    ttl_secs: u64,
    refresh_jti: Option<String>,
) -> doido::Result<()> {
    let expires_at = Utc::now() + Duration::seconds(ttl_secs as i64);
    oauth_access_tokens::ActiveModel {
        jti: Set(jti.to_string()),
        user_id: Set(user_id),
        expires_at: Set(expires_at),
        revoked: Set(false),
        refresh_jti: Set(refresh_jti),
        ..Default::default()
    }
    .insert(db)
    .await?;
    Ok(())
}

pub async fn access_token_valid(
    db: &DatabaseConnection,
    jti: &str,
) -> doido::Result<Option<oauth_access_tokens::Model>> {
    let row = oauth_access_tokens::Entity::find()
        .filter(oauth_access_tokens::Column::Jti.eq(jti))
        .filter(oauth_access_tokens::Column::Revoked.eq(false))
        .one(db)
        .await?;
    Ok(row.filter(|r| r.expires_at >= Utc::now()))
}

pub async fn upsert_mcp_session(
    db: &DatabaseConnection,
    token_jti: &str,
    session_cookie: &str,
) -> doido::Result<()> {
    let now = Utc::now();
    if let Some(existing) = oauth_mcp_sessions::Entity::find()
        .filter(oauth_mcp_sessions::Column::TokenJti.eq(token_jti))
        .one(db)
        .await?
    {
        let mut active: oauth_mcp_sessions::ActiveModel = existing.into();
        active.session_cookie = Set(session_cookie.to_string());
        active.updated_at = Set(now);
        active.update(db).await?;
    } else {
        oauth_mcp_sessions::ActiveModel {
            token_jti: Set(token_jti.to_string()),
            session_cookie: Set(session_cookie.to_string()),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(db)
        .await?;
    }
    Ok(())
}

pub async fn mcp_session_cookie(
    db: &DatabaseConnection,
    token_jti: &str,
) -> doido::Result<Option<String>> {
    Ok(oauth_mcp_sessions::Entity::find()
        .filter(oauth_mcp_sessions::Column::TokenJti.eq(token_jti))
        .one(db)
        .await?
        .map(|s| s.session_cookie))
}

static DEV_CLIENT_READY: OnceLock<()> = OnceLock::new();

pub async fn ensure_dev_client_once(db: &DatabaseConnection) {
    if DEV_CLIENT_READY.get().is_some() {
        return;
    }
    if ensure_dev_client(db).await.is_ok() {
        let _ = DEV_CLIENT_READY.set(());
    }
}

pub async fn ensure_dev_client(db: &DatabaseConnection) -> doido::Result<()> {
    let client_id = std::env::var("MCP_OAUTH_DEV_CLIENT_ID")
        .unwrap_or_else(|_| "quicknfe_mcp_dev".into());
    if find_client(db, &client_id).await?.is_some() {
        return Ok(());
    }
    let uris: Vec<String> = vec![
        "http://127.0.0.1".into(),
        "http://localhost".into(),
        "cursor://anysphere.cursor-mcp/oauth/callback".into(),
    ];
    oauth_clients::ActiveModel {
        client_id: Set(client_id),
        client_secret: Set(None),
        redirect_uris: Set(serde_json::to_string(&uris)?),
        created_at: Set(Utc::now()),
        ..Default::default()
    }
    .insert(db)
    .await?;
    Ok(())
}
