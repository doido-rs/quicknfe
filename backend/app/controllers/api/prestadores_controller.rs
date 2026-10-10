use doido::controller::controller;
use doido::model::sea_orm::{ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, Set};
use serde::Deserialize;

use crate::models::empresa::{ActiveModel as EmpresaActive, Column as EmpresaColumn, Entity as EmpresaEntity};
use crate::services::api_error::{db, id_param, json_body, require_tenant, scoped, to_response};
use crate::services::pagination::{page_params_from_ctx, paginate_select};
use crate::services::dto::EmpresaPublic;
use crate::services::tenant_scope;

#[derive(Debug, Deserialize)]
pub struct PrestadorForm {
    pub cpf_cnpj: String,
    pub razao_social: String,
    pub inscricao_municipal: Option<String>,
    pub inscricao_estadual: Option<String>,
    pub regime_tributario: i32,
    pub codigo_municipio_ibge: String,
    pub uf: String,
}

#[derive(Debug, Deserialize)]
pub struct PrestadorPatchForm {
    pub cpf_cnpj: Option<String>,
    pub razao_social: Option<String>,
    pub inscricao_municipal: Option<String>,
    pub inscricao_estadual: Option<String>,
    pub regime_tributario: Option<i32>,
    pub codigo_municipio_ibge: Option<String>,
    pub uf: Option<String>,
}

pub struct ApiPrestadoresController;

#[controller]
impl ApiPrestadoresController {
    pub async fn index(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let params = crate::api!(to_response(page_params_from_ctx(&ctx)));
        let mut select = EmpresaEntity::find().filter(EmpresaColumn::CompanyId.eq(access.company.id));
        if let Some(ref q) = params.q {
            select = select.filter(
                Condition::any()
                    .add(EmpresaColumn::RazaoSocial.contains(q))
                    .add(EmpresaColumn::CpfCnpj.contains(q)),
            );
        }
        let page = crate::api!(db(
            paginate_select(
                ctx.db(),
                select,
                EmpresaColumn::Id,
                params,
                |m| m.id,
                EmpresaPublic::from,
            )
            .await,
        ));
        ctx.json(page)
    }

    pub async fn show(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let id = crate::api!(id_param(&ctx, "id"));
        let row = crate::api!(scoped(tenant_scope::prestador_for_company(
            ctx.db(),
            access.company.id,
            id,
        )
        .await));
        ctx.json(EmpresaPublic::from(row))
    }

    pub async fn create(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let form: PrestadorForm = crate::api!(json_body(ctx).await);
        let created = crate::api!(db(
            EmpresaActive {
                company_id: Set(access.company.id),
                cpf_cnpj: Set(form.cpf_cnpj),
                razao_social: Set(form.razao_social),
                inscricao_municipal: Set(form.inscricao_municipal),
                inscricao_estadual: Set(form.inscricao_estadual),
                regime_tributario: Set(form.regime_tributario),
                codigo_municipio_ibge: Set(form.codigo_municipio_ibge),
                uf: Set(form.uf),
                ..Default::default()
            }
            .insert(ctx.db())
            .await,
        ));
        ctx.json(EmpresaPublic::from(created))
    }

    pub async fn update(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let id = crate::api!(id_param(&ctx, "id"));
        let existing = crate::api!(scoped(tenant_scope::prestador_for_company(
            ctx.db(),
            access.company.id,
            id,
        )
        .await));
        let form: PrestadorPatchForm = crate::api!(json_body(ctx).await);
        let mut record: EmpresaActive = existing.into();
        if let Some(v) = form.cpf_cnpj {
            record.cpf_cnpj = Set(v);
        }
        if let Some(v) = form.razao_social {
            record.razao_social = Set(v);
        }
        if let Some(v) = form.inscricao_municipal {
            record.inscricao_municipal = Set(Some(v));
        }
        if let Some(v) = form.inscricao_estadual {
            record.inscricao_estadual = Set(Some(v));
        }
        if let Some(v) = form.regime_tributario {
            record.regime_tributario = Set(v);
        }
        if let Some(v) = form.codigo_municipio_ibge {
            record.codigo_municipio_ibge = Set(v);
        }
        if let Some(v) = form.uf {
            record.uf = Set(v);
        }
        let updated = crate::api!(db(record.update(ctx.db()).await));
        ctx.json(EmpresaPublic::from(updated))
    }

    pub async fn destroy(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let id = crate::api!(id_param(&ctx, "id"));
        crate::api!(scoped(tenant_scope::prestador_for_company(
            ctx.db(),
            access.company.id,
            id,
        )
        .await));
        crate::api!(db(EmpresaEntity::delete_by_id(id).exec(ctx.db()).await));
        ctx.status(204)
    }
}
