use doido::controller::controller;
use doido::model::sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::Deserialize;

use crate::models::intermediario::{
    ActiveModel as IntermediarioActive, Column as IntermediarioColumn, Entity as IntermediarioEntity,
};
use crate::services::api_error::{db, id_param, json_body, require_tenant, scoped};
use crate::services::dto::IntermediarioDto;
use crate::services::tenant_scope;

#[derive(Debug, Deserialize)]
pub struct IntermediarioForm {
    pub cpf_cnpj: String,
    pub razao_social: String,
    pub inscricao_municipal: Option<String>,
    pub email: Option<String>,
    pub telefone: Option<String>,
}

pub struct ApiIntermediariosController;

#[controller]
impl ApiIntermediariosController {
    pub async fn index(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let rows = crate::api!(db(
            IntermediarioEntity::find()
                .filter(IntermediarioColumn::EmpresaId.eq(access.prestador.id))
                .all(ctx.db())
                .await,
        ));
        ctx.json(
            rows.into_iter()
                .map(IntermediarioDto::from)
                .collect::<Vec<_>>(),
        )
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
        let created = crate::api!(db(
            IntermediarioActive {
                empresa_id: Set(access.prestador.id),
                cpf_cnpj: Set(form.cpf_cnpj),
                razao_social: Set(form.razao_social),
                inscricao_municipal: Set(form.inscricao_municipal),
                email: Set(form.email),
                telefone: Set(form.telefone),
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
