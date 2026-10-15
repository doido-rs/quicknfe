use doido::controller::controller;
use doido::model::sea_orm::{ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, Set};
use serde::Deserialize;

use crate::models::intermediario::{
    ActiveModel as IntermediarioActive, Column as IntermediarioColumn, Entity as IntermediarioEntity,
};
use crate::services::api_error::{db, id_param, json_body, require_tenant, scoped, to_response};
use crate::services::pagination::{page_params_from_ctx, paginate_select};
use crate::services::dto::IntermediarioDto;
use crate::services::municipio::require_municipio;
use crate::services::tenant_scope;

#[derive(Debug, Deserialize)]
pub struct IntermediarioForm {
    pub cpf_cnpj: String,
    pub razao_social: String,
    pub inscricao_municipal: Option<String>,
    pub email: Option<String>,
    pub telefone: Option<String>,
    pub municipio_id: Option<i64>,
    pub cep: Option<String>,
    pub logradouro: Option<String>,
    pub numero: Option<String>,
    pub bairro: Option<String>,
}

pub struct ApiIntermediariosController;

#[controller]
impl ApiIntermediariosController {
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
            IntermediarioEntity::find().filter(IntermediarioColumn::EmpresaId.eq(prestador_id));
        if let Some(ref q) = params.q {
            select = select.filter(
                Condition::any()
                    .add(IntermediarioColumn::RazaoSocial.contains(q))
                    .add(IntermediarioColumn::CpfCnpj.contains(q)),
            );
        }
        let page = crate::api!(db(
            paginate_select(
                ctx.db(),
                select,
                IntermediarioColumn::Id,
                params,
                |m| m.id,
                IntermediarioDto::from,
            )
            .await,
        ));
        ctx.json(page)
    }

    pub async fn show(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let id = crate::api!(id_param(&ctx, "id"));
        let row = crate::api!(scoped(
            tenant_scope::intermediario_for_tenant(ctx.db(), access.prestador.id, id).await,
        ));
        ctx.json(IntermediarioDto::from(row))
    }

    pub async fn create(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let form: IntermediarioForm = crate::api!(json_body(ctx).await);
        if let Some(mid) = form.municipio_id {
            crate::api!(to_response(require_municipio(ctx.db(), mid).await));
        }
        let created = crate::api!(db(
            IntermediarioActive {
                empresa_id: Set(access.prestador.id),
                cpf_cnpj: Set(form.cpf_cnpj),
                razao_social: Set(form.razao_social),
                inscricao_municipal: Set(form.inscricao_municipal),
                email: Set(form.email),
                telefone: Set(form.telefone),
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
        ctx.json(IntermediarioDto::from(created))
    }

    pub async fn update(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let id = crate::api!(id_param(&ctx, "id"));
        let existing = crate::api!(scoped(
            tenant_scope::intermediario_for_tenant(ctx.db(), access.prestador.id, id).await,
        ));
        let form: IntermediarioForm = crate::api!(json_body(ctx).await);
        let mut record: IntermediarioActive = existing.into();
        record.cpf_cnpj = Set(form.cpf_cnpj);
        record.razao_social = Set(form.razao_social);
        record.inscricao_municipal = Set(form.inscricao_municipal);
        record.email = Set(form.email);
        record.telefone = Set(form.telefone);
        if let Some(mid) = form.municipio_id {
            crate::api!(to_response(require_municipio(ctx.db(), mid).await));
            record.municipio_id = Set(Some(mid));
        }
        record.cep = Set(form.cep);
        record.logradouro = Set(form.logradouro);
        record.numero = Set(form.numero);
        record.bairro = Set(form.bairro);
        let updated = crate::api!(db(record.update(ctx.db()).await));
        ctx.json(IntermediarioDto::from(updated))
    }

    pub async fn destroy(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let id = crate::api!(id_param(&ctx, "id"));
        crate::api!(scoped(
            tenant_scope::intermediario_for_tenant(ctx.db(), access.prestador.id, id).await,
        ));
        crate::api!(db(IntermediarioEntity::delete_by_id(id).exec(ctx.db()).await));
        ctx.status(204)
    }
}
