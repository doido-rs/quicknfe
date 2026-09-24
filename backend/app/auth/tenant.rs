//! Empresa ativa do usuário autenticado (multi-tenant).

use doido::controller::axum::extract::FromRequestParts;
use doido::controller::axum::http::request::Parts;
use doido::controller::axum::http::StatusCode;
use doido::controller::axum::response::{IntoResponse, Response};
use doido::controller::secret::key_base;
use doido::controller::session::{EncryptedCookieSessionStore, Session};
use doido::model::sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use doido_auth::extractors::CurrentUser;
use doido_auth::session::SESSION_COOKIE;

use crate::models::empresa::Model as Empresa;
use crate::models::empresa_usuario::{Column as MembroColumn, Entity as MembroEntity};
use crate::models::user::Model as User;

/// Chave na sessão criptografada (`_doido_session`) para a empresa ativa.
pub const EMPRESA_ID_KEY: &str = "empresa_id";

#[derive(Debug, Clone)]
pub struct TenantAccess {
    pub user: User,
    pub empresa: Empresa,
    pub papel: String,
}

#[derive(Debug)]
pub enum TenantError {
    Unauthorized,
    Forbidden,
    NoTenantSelected,
    Internal(String),
}

impl IntoResponse for TenantError {
    fn into_response(self) -> Response {
        let status = match self {
            TenantError::Unauthorized => StatusCode::UNAUTHORIZED,
            TenantError::Forbidden | TenantError::NoTenantSelected => StatusCode::FORBIDDEN,
            TenantError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}

impl std::fmt::Display for TenantError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TenantError::Unauthorized => write!(f, "authentication required"),
            TenantError::Forbidden => write!(f, "no access to this empresa"),
            TenantError::NoTenantSelected => write!(f, "select an empresa for this session"),
            TenantError::Internal(msg) => write!(f, "{msg}"),
        }
    }
}

pub async fn membership(
    db: &DatabaseConnection,
    user_id: i64,
    empresa_id: i64,
) -> doido::Result<Option<String>> {
    Ok(MembroEntity::find()
        .filter(MembroColumn::UserId.eq(user_id))
        .filter(MembroColumn::EmpresaId.eq(empresa_id))
        .one(db)
        .await?
        .map(|m| m.papel))
}

pub async fn user_empresa_ids(db: &DatabaseConnection, user_id: i64) -> doido::Result<Vec<i64>> {
    let rows = MembroEntity::find()
        .filter(MembroColumn::UserId.eq(user_id))
        .all(db)
        .await?;
    Ok(rows.into_iter().map(|r| r.empresa_id).collect())
}

fn session_from_parts(parts: &Parts) -> Option<Session> {
    let header_value = parts.headers.get(doido::controller::axum::http::header::COOKIE)?.to_str().ok()?;
    let raw = header_value
        .split(';')
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(k, _)| *k == SESSION_COOKIE)
        .map(|(_, v)| v.to_string())?;
    let store = EncryptedCookieSessionStore::new(key_base());
    store.decode(&raw)
}

pub fn empresa_id_from_session(parts: &Parts) -> Option<i64> {
    let session = session_from_parts(parts)?;
    session.get(EMPRESA_ID_KEY)
}

pub fn set_empresa_in_session(session: &mut Session, empresa_id: i64) {
    session.set(EMPRESA_ID_KEY, empresa_id);
}

/// Exige usuário autenticado + empresa ativa na sessão com vínculo válido.
pub struct CurrentTenant(pub TenantAccess);

impl<S> FromRequestParts<S> for CurrentTenant
where
    S: Send + Sync,
{
    type Rejection = TenantError;

    fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            let CurrentUser(user) = CurrentUser::<User>::from_request_parts(parts, state)
                .await
                .map_err(|_| TenantError::Unauthorized)?;

            let empresa_id = empresa_id_from_session(parts).ok_or(TenantError::NoTenantSelected)?;

            let db = doido_auth::state::global().db.clone();
            let Some(papel) = membership(&db, user.id, empresa_id)
                .await
                .map_err(|e| TenantError::Internal(e.to_string()))?
            else {
                return Err(TenantError::Forbidden);
            };

            let Some(empresa) = crate::models::empresa::Entity::find_by_id(empresa_id)
                .one(&db)
                .await
                .map_err(|e| TenantError::Internal(e.to_string()))?
            else {
                return Err(TenantError::Forbidden);
            };

            Ok(CurrentTenant(TenantAccess {
                user,
                empresa,
                papel,
            }))
        }
    }
}
