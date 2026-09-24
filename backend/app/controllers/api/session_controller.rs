use doido::controller::controller;
use doido::model::sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::auth::tenant::{self, EMPRESA_ID_KEY};
use crate::models::empresa::Entity as EmpresaEntity;
use crate::models::empresa_usuario::{Column as MembroColumn, Entity as MembroEntity};
use crate::services::api_error::{ApiErrExt, ApiError};
use crate::services::dto::{EmpresaMembership, EmpresaPublic, MeResponse, UserPublic};

pub struct ApiSessionController;

#[controller]
impl ApiSessionController {
    /// GET /api/me
    pub async fn me(
        mut ctx: doido::controller::Context,
    ) -> doido::controller::Response {
        let user = tenant::user_from_ctx(ctx).await.api()?;
        let db = ctx.db();
        let membros = MembroEntity::find()
            .filter(MembroColumn::UserId.eq(user.id))
            .all(db)
            .await
            .map_err(|e| ApiError::Internal(e.to_string())).api()?;

        let mut empresas = Vec::with_capacity(membros.len());
        for m in membros {
            if let Some(empresa) = EmpresaEntity::find_by_id(m.empresa_id)
                .one(db)
                .await
                .map_err(|e| ApiError::Internal(e.to_string())).api()?
            {
                empresas.push(EmpresaMembership {
                    empresa: EmpresaPublic::from(empresa),
                    papel: m.papel,
                });
            }
        }

        let active_id: Option<i64> = ctx.session().get(EMPRESA_ID_KEY);
        let (empresa_ativa, papel) = if let Some(eid) = active_id {
            let membership = tenant::membership(db, user.id, eid)
                .await
                .map_err(|e| ApiError::Internal(e.to_string())).api()?;
            if let Some(p) = membership {
                let emp = EmpresaEntity::find_by_id(eid)
                    .one(db)
                    .await
                    .map_err(|e| ApiError::Internal(e.to_string())).api()?;
                (
                    emp.map(EmpresaPublic::from),
                    Some(p),
                )
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        ctx.json(MeResponse {
            user: UserPublic::from(user),
            empresas,
            empresa_ativa,
            papel,
        })
    }
}
