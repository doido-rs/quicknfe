use doido::controller::controller;
use doido::model::sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::auth::tenant::{self, EMPRESA_ID_KEY};
use crate::models::empresa::Entity as EmpresaEntity;
use crate::models::empresa_usuario::{Column as MembroColumn, Entity as MembroEntity};
use crate::services::api_error::{db, require_user};
use crate::services::dto::{EmpresaMembership, EmpresaPublic, MeResponse, UserPublic};

pub struct ApiSessionController;

#[controller]
impl ApiSessionController {
    pub async fn me(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let user = crate::api!(require_user(ctx).await);
        let conn = ctx.db();
        let membros = crate::api!(db(
            MembroEntity::find()
                .filter(MembroColumn::UserId.eq(user.id))
                .all(conn)
                .await,
        ));

        let mut empresas = Vec::with_capacity(membros.len());
        for m in membros {
            if let Some(empresa) = crate::api!(db(EmpresaEntity::find_by_id(m.empresa_id).one(conn).await)) {
                empresas.push(EmpresaMembership {
                    empresa: EmpresaPublic::from(empresa),
                    papel: m.papel,
                });
            }
        }

        let active_id: Option<i64> = ctx.session().get(EMPRESA_ID_KEY);
        let (empresa_ativa, papel) = if let Some(eid) = active_id {
            let membership = crate::api!(db(tenant::membership(conn, user.id, eid).await));
            if let Some(p) = membership {
                let emp = crate::api!(db(EmpresaEntity::find_by_id(eid).one(conn).await));
                (emp.map(EmpresaPublic::from), Some(p))
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
