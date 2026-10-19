use doido::controller::controller;
use doido::model::sea_orm::{ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, Set};
use serde::Deserialize;

use crate::models::conta_fiscal::{
    ActiveModel as ContaFiscalActive, Column as ContaFiscalColumn, Entity as ContaFiscalEntity,
};
use crate::services::api_error::{
    db, id_param, json_body, require_tenant, scoped, to_response, ApiError,
};
use doido::controller::axum::response::IntoResponse;
use crate::services::conta_fiscal::validar_conta_fiscal;
use crate::services::dto::ContaFiscalDto;
use crate::services::pagination::{page_params_from_ctx, paginate_select};
use crate::services::tenant_scope;

#[derive(Debug, Deserialize)]
pub struct ContaFiscalForm {
    pub codigo: String,
    pub descricao: Option<String>,
    pub cna_id: i64,
    pub codigo_tributacao_nacional: Option<String>,
    pub codigo_tributacao_municipal: String,
}

pub struct ApiContasFiscaisController;

#[controller]
impl ApiContasFiscaisController {
    pub async fn index(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let prestador_id = match crate::api!(to_response(tenant_scope::empresa_id_from_list_query(&ctx)))
        {
            Some(id) => {
                crate::api!(scoped(
                    tenant_scope::prestador_for_company(ctx.db(), access.company.id, id).await,
                ));
                id
            }
            None => access.prestador.id,
        };
        let params = crate::api!(to_response(page_params_from_ctx(&ctx)));
        let mut select =
            ContaFiscalEntity::find().filter(ContaFiscalColumn::EmpresaId.eq(prestador_id));
        if let Some(ref q) = params.q {
            select = select.filter(
                Condition::any()
                    .add(ContaFiscalColumn::Codigo.contains(q))
                    .add(ContaFiscalColumn::Descricao.contains(q))
                    .add(ContaFiscalColumn::CodigoTributacaoNacional.contains(q))
                    .add(ContaFiscalColumn::CodigoTributacaoMunicipal.contains(q)),
            );
        }
        let page = crate::api!(db(
            paginate_select(
                ctx.db(),
                select,
                ContaFiscalColumn::Id,
                params,
                |m| m.id,
                ContaFiscalDto::from,
            )
            .await,
        ));
        ctx.json(page)
    }

    pub async fn show(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let id = crate::api!(id_param(&ctx, "id"));
        let row = crate::api!(scoped(
            tenant_scope::conta_fiscal_for_tenant(ctx.db(), access.prestador.id, id).await,
        ));
        ctx.json(ContaFiscalDto::from(row))
    }

    pub async fn create(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let form: ContaFiscalForm = crate::api!(json_body(ctx).await);
        if form.codigo.trim().is_empty() {
            return ApiError::Validation("Código da conta fiscal é obrigatório".into())
                .into_response();
        }
        let validated = crate::api!(to_response(
            validar_conta_fiscal(
                ctx.db(),
                access.prestador.id,
                form.cna_id,
                form.codigo_tributacao_nacional.as_deref(),
                &form.codigo_tributacao_municipal,
            )
            .await,
        ));
        let created = crate::api!(db(
            ContaFiscalActive {
                empresa_id: Set(access.prestador.id),
                codigo: Set(form.codigo.trim().to_string()),
                descricao: Set(form.descricao.filter(|s| !s.trim().is_empty())),
                cna_id: Set(validated.cna_id),
                codigo_tributacao_nacional: Set(validated.codigo_tributacao_nacional),
                codigo_tributacao_municipal: Set(validated.codigo_tributacao_municipal),
                ..Default::default()
            }
            .insert(ctx.db())
            .await,
        ));
        ctx.json(ContaFiscalDto::from(created))
    }

    pub async fn update(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let id = crate::api!(id_param(&ctx, "id"));
        let existing = crate::api!(scoped(
            tenant_scope::conta_fiscal_for_tenant(ctx.db(), access.prestador.id, id).await,
        ));
        let form: ContaFiscalForm = crate::api!(json_body(ctx).await);
        if form.codigo.trim().is_empty() {
            return ApiError::Validation("Código da conta fiscal é obrigatório".into())
                .into_response();
        }
        let validated = crate::api!(to_response(
            validar_conta_fiscal(
                ctx.db(),
                access.prestador.id,
                form.cna_id,
                form.codigo_tributacao_nacional.as_deref(),
                &form.codigo_tributacao_municipal,
            )
            .await,
        ));
        let mut record: ContaFiscalActive = existing.into();
        record.codigo = Set(form.codigo.trim().to_string());
        record.descricao = Set(form.descricao.filter(|s| !s.trim().is_empty()));
        record.cna_id = Set(validated.cna_id);
        record.codigo_tributacao_nacional = Set(validated.codigo_tributacao_nacional);
        record.codigo_tributacao_municipal = Set(validated.codigo_tributacao_municipal);
        let updated = crate::api!(db(record.update(ctx.db()).await));
        ctx.json(ContaFiscalDto::from(updated))
    }

    pub async fn destroy(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let id = crate::api!(id_param(&ctx, "id"));
        crate::api!(scoped(
            tenant_scope::conta_fiscal_for_tenant(ctx.db(), access.prestador.id, id).await,
        ));
        crate::api!(db(ContaFiscalEntity::delete_by_id(id).exec(ctx.db()).await));
        ctx.status(204)
    }
}
