use doido::controller::controller;
use doido::model::sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use chrono::Utc;

use crate::auth::tenant::{self, COMPANY_ID_KEY, PRESTADOR_ID_KEY};
use crate::models::company::Entity as CompanyEntity;
use crate::models::company_usuario::{Column as MembroColumn, Entity as MembroEntity};
use crate::models::empresa::{Column as EmpresaColumn, Entity as EmpresaEntity};
use crate::models::user::ActiveModel as UserActive;
use crate::services::api_error::{db, json_body, require_user, ApiError};
use crate::services::dto::{
    empresas_public, empresa_public, CompanyMembership, CompanyPublic, MeResponse, UpdateMeForm,
    UserPublic,
};
use crate::services::api_error::to_response;

fn valid_date_format(s: &str) -> bool {
    matches!(s, "DMY" | "MDY" | "YMD")
}

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

        let mut companies = Vec::with_capacity(membros.len());
        for m in membros {
            if let Some(company) =
                crate::api!(db(CompanyEntity::find_by_id(m.company_id).one(conn).await))
            {
                companies.push(CompanyMembership {
                    company: CompanyPublic::from(company),
                    papel: m.papel,
                });
            }
        }

        let active_company_id: Option<i64> = ctx.session().get(COMPANY_ID_KEY);
        let active_prestador_id: Option<i64> = ctx.session().get(PRESTADOR_ID_KEY);

        let (company_ativa, papel) = if let Some(cid) = active_company_id {
            let membership = crate::api!(db(tenant::membership(conn, user.id, cid).await));
            if let Some(p) = membership {
                let company = crate::api!(db(CompanyEntity::find_by_id(cid).one(conn).await));
                (company.map(CompanyPublic::from), Some(p))
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        let prestadores = if let Some(cid) = company_ativa.as_ref().map(|c| c.id).or(active_company_id)
        {
            let rows = crate::api!(db(
                EmpresaEntity::find()
                    .filter(EmpresaColumn::CompanyId.eq(cid))
                    .all(conn)
                    .await,
            ));
            crate::api!(to_response(empresas_public(conn, rows).await))
        } else {
            vec![]
        };

        let prestador_ativo = if let (Some(cid), Some(pid)) = (active_company_id, active_prestador_id)
        {
            if let Some(row) = crate::api!(db(
                EmpresaEntity::find_by_id(pid)
                    .filter(EmpresaColumn::CompanyId.eq(cid))
                    .one(conn)
                    .await,
            )) {
                Some(crate::api!(to_response(empresa_public(conn, row).await)))
            } else {
                None
            }
        } else {
            None
        };

        ctx.json(MeResponse {
            user: UserPublic::from(user),
            companies,
            company_ativa,
            prestadores,
            prestador_ativo,
            papel,
        })
    }

    pub async fn update_me(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let user = crate::api!(require_user(ctx).await);
        let form: UpdateMeForm = crate::api!(json_body(ctx).await);
        if let Some(df) = form.date_format {
            if !valid_date_format(&df) {
                crate::api!(to_response(Err::<(), ApiError>(ApiError::Validation(
                    "date_format must be DMY, MDY, or YMD".into(),
                ))));
            }
            let mut record: UserActive = user.into();
            record.date_format = Set(df);
            record.updated_at = Set(Utc::now());
            let updated = crate::api!(db(record.update(ctx.db()).await));
            return ctx.json(UserPublic::from(updated));
        }
        ctx.json(UserPublic::from(user))
    }
}
