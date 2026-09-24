use doido::controller::controller;
use doido_auth::session::USER_ID_KEY;
use doido_auth::AuthUser;
use serde::Deserialize;

use crate::auth::tenant::{membership, set_empresa_in_session, TenantError};
use crate::models::user::Model as User;

#[derive(Deserialize)]
pub struct SwitchTenantForm {
    pub empresa_id: i64,
}

pub struct TenantController;

#[controller]
impl TenantController {
    /// POST /tenant/empresa — body `{ "empresa_id": 1 }` define a empresa ativa na sessão.
    pub async fn switch(
        mut ctx: doido::controller::Context,
    ) -> Result<doido::controller::Response, TenantError> {
        let form: SwitchTenantForm = ctx
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

        if membership(db, user.id, form.empresa_id)
            .await
            .map_err(|e| TenantError::Internal(e.to_string()))?
            .is_none()
        {
            return Err(TenantError::Forbidden);
        }

        set_empresa_in_session(ctx.session(), form.empresa_id);
        Ok(ctx.status(204))
    }
}
