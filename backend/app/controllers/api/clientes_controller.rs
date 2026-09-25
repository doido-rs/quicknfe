use doido::controller::controller;
use doido::model::sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::Deserialize;

use crate::models::cliente::{ActiveModel as ClienteActive, Column as ClienteColumn, Entity as ClienteEntity};
use crate::services::api_error::{db, id_param, json_body, require_tenant, scoped};
use crate::services::dto::ClienteDto;
use crate::services::tenant_scope;

#[derive(Debug, Deserialize)]
pub struct ClienteForm {
    pub cpf_cnpj: String,
    pub razao_social: String,
    pub inscricao_municipal: Option<String>,
    pub inscricao_estadual: Option<String>,
    pub logradouro: Option<String>,
    pub numero: Option<String>,
    pub bairro: Option<String>,
    pub codigo_municipio_ibge: Option<String>,
    pub uf: Option<String>,
    pub email: Option<String>,
    pub telefone: Option<String>,
}

pub struct ApiClientesController;

#[controller]
impl ApiClientesController {
    pub async fn index(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let rows = crate::api!(db(
            ClienteEntity::find()
                .filter(ClienteColumn::EmpresaId.eq(access.empresa.id))
                .all(ctx.db())
                .await,
        ));
        ctx.json(
            rows.into_iter()
                .map(ClienteDto::from)
                .collect::<Vec<_>>(),
        )
    }

    pub async fn show(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let id = crate::api!(id_param(&ctx, "id"));
        let row = crate::api!(scoped(tenant_scope::cliente_for_tenant(ctx.db(), access.empresa.id, id).await));
        ctx.json(ClienteDto::from(row))
    }

    pub async fn create(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let form: ClienteForm = crate::api!(json_body(ctx).await);
        let created = crate::api!(db(
            ClienteActive {
                empresa_id: Set(access.empresa.id),
                cpf_cnpj: Set(form.cpf_cnpj),
                razao_social: Set(form.razao_social),
                inscricao_municipal: Set(form.inscricao_municipal),
                inscricao_estadual: Set(form.inscricao_estadual),
                logradouro: Set(form.logradouro),
                numero: Set(form.numero),
                bairro: Set(form.bairro),
                codigo_municipio_ibge: Set(form.codigo_municipio_ibge),
                uf: Set(form.uf),
                email: Set(form.email),
                telefone: Set(form.telefone),
                ..Default::default()
            }
            .insert(ctx.db())
            .await,
        ));
        ctx.json(ClienteDto::from(created))
    }

    pub async fn update(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let id = crate::api!(id_param(&ctx, "id"));
        let existing = crate::api!(scoped(tenant_scope::cliente_for_tenant(ctx.db(), access.empresa.id, id).await));
        let form: ClienteForm = crate::api!(json_body(ctx).await);
        let mut record: ClienteActive = existing.into();
        record.cpf_cnpj = Set(form.cpf_cnpj);
        record.razao_social = Set(form.razao_social);
        record.inscricao_municipal = Set(form.inscricao_municipal);
        record.inscricao_estadual = Set(form.inscricao_estadual);
        record.logradouro = Set(form.logradouro);
        record.numero = Set(form.numero);
        record.bairro = Set(form.bairro);
        record.codigo_municipio_ibge = Set(form.codigo_municipio_ibge);
        record.uf = Set(form.uf);
        record.email = Set(form.email);
        record.telefone = Set(form.telefone);
        let updated = crate::api!(db(record.update(ctx.db()).await));
        ctx.json(ClienteDto::from(updated))
    }

    pub async fn destroy(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let id = crate::api!(id_param(&ctx, "id"));
        crate::api!(scoped(tenant_scope::cliente_for_tenant(ctx.db(), access.empresa.id, id).await));
        crate::api!(db(ClienteEntity::delete_by_id(id).exec(ctx.db()).await));
        ctx.status(204)
    }
}
