use doido::controller::controller;
use doido::model::sea_orm::{ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, Set};
use serde::Deserialize;

use crate::models::empresa::{ActiveModel as EmpresaActive, Column as EmpresaColumn, Entity as EmpresaEntity};
use crate::models::regime_tributario::RegimeTributario;
use crate::services::api_error::{db, id_param, json_body, require_tenant, scoped, to_response};
use crate::services::dto::{empresa_public, empresas_public};
use crate::services::municipio::require_municipio;
use crate::services::pagination::{page_params_from_ctx, paginate_select, PageResponse};
use crate::services::tenant_scope;

#[derive(Debug, Deserialize)]
pub struct PrestadorForm {
    pub cpf_cnpj: String,
    pub razao_social: String,
    pub inscricao_municipal: Option<String>,
    pub inscricao_estadual: Option<String>,
    pub regime_tributario: RegimeTributario,
    pub municipio_id: i64,
    pub cep: Option<String>,
    pub logradouro: Option<String>,
    pub numero: Option<String>,
    pub bairro: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PrestadorPatchForm {
    pub cpf_cnpj: Option<String>,
    pub razao_social: Option<String>,
    pub inscricao_municipal: Option<String>,
    pub inscricao_estadual: Option<String>,
    pub regime_tributario: Option<RegimeTributario>,
    pub municipio_id: Option<i64>,
    pub cep: Option<String>,
    pub logradouro: Option<String>,
    pub numero: Option<String>,
    pub bairro: Option<String>,
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
                |m| m,
            )
            .await,
        ));
        let items = crate::api!(to_response(empresas_public(ctx.db(), page.items).await));
        ctx.json(PageResponse {
            items,
            next_page_token: page.next_page_token,
        })
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
        let dto = crate::api!(to_response(empresa_public(ctx.db(), row).await));
        ctx.json(dto)
    }

    pub async fn create(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let form: PrestadorForm = crate::api!(json_body(ctx).await);
        crate::api!(to_response(require_municipio(ctx.db(), form.municipio_id).await));
        let created = crate::api!(db(
            EmpresaActive {
                company_id: Set(access.company.id),
                cpf_cnpj: Set(form.cpf_cnpj),
                razao_social: Set(form.razao_social),
                inscricao_municipal: Set(form.inscricao_municipal),
                inscricao_estadual: Set(form.inscricao_estadual),
                regime_tributario: Set(form.regime_tributario.as_db_code()),
                municipio_id: Set(form.municipio_id),
                cep: Set(form.cep),
                logradouro: Set(form.logradouro),
                numero: Set(form.numero),
                bairro: Set(form.bairro),
                ..Default::default()
            }
            .insert(ctx.db())
            .await,
        ));
        let dto = crate::api!(to_response(empresa_public(ctx.db(), created).await));
        ctx.json(dto)
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
        if let Some(municipio_id) = form.municipio_id {
            crate::api!(to_response(require_municipio(ctx.db(), municipio_id).await));
        }
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
            record.regime_tributario = Set(v.as_db_code());
        }
        if let Some(v) = form.municipio_id {
            record.municipio_id = Set(v);
        }
        if let Some(v) = form.cep {
            record.cep = Set(Some(v));
        }
        if let Some(v) = form.logradouro {
            record.logradouro = Set(Some(v));
        }
        if let Some(v) = form.numero {
            record.numero = Set(Some(v));
        }
        if let Some(v) = form.bairro {
            record.bairro = Set(Some(v));
        }
        let updated = crate::api!(db(record.update(ctx.db()).await));
        let dto = crate::api!(to_response(empresa_public(ctx.db(), updated).await));
        ctx.json(dto)
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
