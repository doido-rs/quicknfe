//! Company (tenant) + prestador ativo (`empresas`) na sessão.

use doido::controller::axum::http::StatusCode;
use doido::controller::axum::response::{IntoResponse, Response};
use doido::controller::session::Session;
use serde_json::Value;
use doido::model::sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use doido_auth::session::USER_ID_KEY;
use doido_auth::AuthUser;

use crate::models::company::Model as Company;
use crate::models::company_usuario::{Column as MembroColumn, Entity as MembroEntity};
use crate::models::empresa::{Column as EmpresaColumn, Entity as EmpresaEntity, Model as Empresa};
use crate::models::papel_usuario::PapelUsuario;
use crate::models::user::Model as User;
use crate::services::entity_enums::papel_from_db;

/// Tenant organizacional (company).
pub const COMPANY_ID_KEY: &str = "company_id";
/// Prestador de serviços ativo (`empresas.id`).
pub const PRESTADOR_ID_KEY: &str = "empresa_id";

#[derive(Debug, Clone)]
pub struct TenantAccess {
    pub user: User,
    pub company: Company,
    pub prestador: Empresa,
    pub papel: PapelUsuario,
}

#[derive(Debug)]
pub enum TenantError {
    Unauthorized,
    Forbidden,
    NoTenantSelected,
    NoPrestadorSelected,
    Internal(String),
}

impl IntoResponse for TenantError {
    fn into_response(self) -> Response {
        let status = match self {
            TenantError::Unauthorized => StatusCode::UNAUTHORIZED,
            TenantError::Forbidden | TenantError::NoTenantSelected | TenantError::NoPrestadorSelected => {
                StatusCode::FORBIDDEN
            }
            TenantError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}

impl std::fmt::Display for TenantError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TenantError::Unauthorized => write!(f, "authentication required"),
            TenantError::Forbidden => write!(f, "no access to this company"),
            TenantError::NoTenantSelected => write!(f, "select a company for this session"),
            TenantError::NoPrestadorSelected => write!(f, "select a prestador for this session"),
            TenantError::Internal(msg) => write!(f, "{msg}"),
        }
    }
}

pub async fn membership(
    db: &DatabaseConnection,
    user_id: i64,
    company_id: i64,
) -> doido::Result<Option<PapelUsuario>> {
    let row = MembroEntity::find()
        .filter(MembroColumn::UserId.eq(user_id))
        .filter(MembroColumn::CompanyId.eq(company_id))
        .one(db)
        .await?;
    Ok(match row {
        Some(m) => Some(papel_from_db(m.papel).map_err(|e| {
            doido_core::anyhow::anyhow!("{e}")
        })?),
        None => None,
    })
}

pub async fn user_company_ids(db: &DatabaseConnection, user_id: i64) -> doido::Result<Vec<i64>> {
    let rows = MembroEntity::find()
        .filter(MembroColumn::UserId.eq(user_id))
        .all(db)
        .await?;
    Ok(rows.into_iter().map(|r| r.company_id).collect())
}

pub fn set_company_in_session(session: &mut Session, company_id: i64) {
    session.set(COMPANY_ID_KEY, company_id);
}

pub fn set_prestador_in_session(session: &mut Session, prestador_id: i64) {
    session.set(PRESTADOR_ID_KEY, prestador_id);
}

pub fn clear_prestador_in_session(session: &mut Session) {
    if let Value::Object(map) = &mut session.data {
        map.remove(PRESTADOR_ID_KEY);
    }
}

pub async fn load_prestador_for_company(
    db: &DatabaseConnection,
    company_id: i64,
    prestador_id: i64,
) -> Result<Empresa, TenantError> {
    let Some(prestador) = EmpresaEntity::find_by_id(prestador_id)
        .filter(EmpresaColumn::CompanyId.eq(company_id))
        .one(db)
        .await
        .map_err(|e| TenantError::Internal(e.to_string()))?
    else {
        return Err(TenantError::Forbidden);
    };
    Ok(prestador)
}

/// Usuário autenticado a partir da sessão do [`Context`].
pub async fn user_from_ctx(ctx: &mut doido::controller::Context) -> Result<User, TenantError> {
    let user_id: i64 = ctx
        .session()
        .get(USER_ID_KEY)
        .ok_or(TenantError::Unauthorized)?;
    let db = ctx.db();
    User::find_by_id(db, user_id)
        .await
        .map_err(|e| TenantError::Internal(e.to_string()))?
        .ok_or(TenantError::Unauthorized)
}

async fn resolve_tenant_ids(
    db: &DatabaseConnection,
    session: &Session,
) -> Result<(i64, i64), TenantError> {
    let company_id = session.get::<i64>(COMPANY_ID_KEY);
    let prestador_id = session.get::<i64>(PRESTADOR_ID_KEY);

    match (company_id, prestador_id) {
        (Some(c), Some(p)) => Ok((c, p)),
        (None, Some(p)) => {
            let Some(prestador) = EmpresaEntity::find_by_id(p)
                .one(db)
                .await
                .map_err(|e| TenantError::Internal(e.to_string()))?
            else {
                return Err(TenantError::Forbidden);
            };
            Ok((prestador.company_id, p))
        }
        (Some(_), None) => Err(TenantError::NoPrestadorSelected),
        (None, None) => Err(TenantError::NoTenantSelected),
    }
}

/// Company + prestador ativo a partir da sessão.
pub async fn tenant_from_ctx(
    ctx: &mut doido::controller::Context,
) -> Result<TenantAccess, TenantError> {
    let user = user_from_ctx(ctx).await?;
    let db = ctx.db();
    let (company_id, prestador_id) = resolve_tenant_ids(db, ctx.session()).await?;

    let Some(papel) = membership(db, user.id, company_id)
        .await
        .map_err(|e| TenantError::Internal(e.to_string()))?
    else {
        return Err(TenantError::Forbidden);
    };

    let Some(company) = crate::models::company::Entity::find_by_id(company_id)
        .one(db)
        .await
        .map_err(|e| TenantError::Internal(e.to_string()))?
    else {
        return Err(TenantError::Forbidden);
    };

    let prestador = load_prestador_for_company(db, company_id, prestador_id).await?;

    Ok(TenantAccess {
        user,
        company,
        prestador,
        papel,
    })
}
