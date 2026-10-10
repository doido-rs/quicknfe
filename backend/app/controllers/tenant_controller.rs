use doido::controller::controller;
use doido::model::sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use doido_auth::session::USER_ID_KEY;
use doido_auth::AuthUser;
use serde::Deserialize;

use crate::auth::tenant::{
    clear_prestador_in_session, load_prestador_for_company, membership, set_company_in_session,
    set_prestador_in_session, TenantError,
};
use crate::models::empresa::{Column as EmpresaColumn, Entity as EmpresaEntity};
use crate::models::user::Model as User;

#[derive(Deserialize)]
pub struct SwitchCompanyForm {
    pub company_id: i64,
}

#[derive(Deserialize)]
pub struct SwitchPrestadorForm {
    pub empresa_id: i64,
}

pub struct TenantController;

#[controller]
impl TenantController {
    /// POST /tenant/company — define a company (tenant) ativa na sessão.
    pub async fn switch_company(
        mut ctx: doido::controller::Context,
    ) -> Result<doido::controller::Response, TenantError> {
        let form: SwitchCompanyForm = ctx
            .body_json()
            .await
            .map_err(|e| TenantError::Internal(e.to_string()))?;

        let user_id: i64 = ctx
            .session()
            .get(USER_ID_KEY)
            .ok_or(TenantError::Unauthorized)?;

        let db = ctx.db();
        let Some(user) = User::find_by_id(db, user_id)
            .await
            .map_err(|e| TenantError::Internal(e.to_string()))?
        else {
            return Err(TenantError::Unauthorized);
        };

        if membership(db, user.id, form.company_id)
            .await
            .map_err(|e| TenantError::Internal(e.to_string()))?
            .is_none()
        {
            return Err(TenantError::Forbidden);
        }

        set_company_in_session(ctx.session(), form.company_id);
        clear_prestador_in_session(ctx.session());

        let prestadores = EmpresaEntity::find()
            .filter(EmpresaColumn::CompanyId.eq(form.company_id))
            .all(db)
            .await
            .map_err(|e| TenantError::Internal(e.to_string()))?;

        if prestadores.len() == 1 {
            set_prestador_in_session(ctx.session(), prestadores[0].id);
        }

        Ok(ctx.status(204))
    }

    /// POST /tenant/prestador — body `{ "empresa_id": 1 }` define o prestador ativo.
    pub async fn switch_prestador(
        mut ctx: doido::controller::Context,
    ) -> Result<doido::controller::Response, TenantError> {
        let form: SwitchPrestadorForm = ctx
            .body_json()
            .await
            .map_err(|e| TenantError::Internal(e.to_string()))?;

        let user_id: i64 = ctx
            .session()
            .get(USER_ID_KEY)
            .ok_or(TenantError::Unauthorized)?;

        let db = ctx.db();
        let Some(user) = User::find_by_id(db, user_id)
            .await
            .map_err(|e| TenantError::Internal(e.to_string()))?
        else {
            return Err(TenantError::Unauthorized);
        };

        let company_id = ctx
            .session()
            .get(crate::auth::tenant::COMPANY_ID_KEY)
            .ok_or(TenantError::NoTenantSelected)?;

        if membership(db, user.id, company_id)
            .await
            .map_err(|e| TenantError::Internal(e.to_string()))?
            .is_none()
        {
            return Err(TenantError::Forbidden);
        }

        load_prestador_for_company(db, company_id, form.empresa_id).await?;

        set_prestador_in_session(ctx.session(), form.empresa_id);
        Ok(ctx.status(204))
    }
}
