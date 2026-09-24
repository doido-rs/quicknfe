use doido::controller::controller;
use doido::model::sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::Deserialize;

use crate::auth::tenant;
use crate::models::cliente::{ActiveModel as ClienteActive, Column as ClienteColumn, Entity as ClienteEntity};
use crate::services::api_error::{parse_id, ApiErrExt, ApiError};
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
    pub async fn index(
        mut ctx: doido::controller::Context,
    ) -> doido::controller::Response {
        let access = tenant::tenant_from_ctx(ctx).await.api()?;
        let rows = ClienteEntity::find()
            .filter(ClienteColumn::EmpresaId.eq(access.empresa.id))
            .all(ctx.db())
            .await
            .map_err(|e| ApiError::Internal(e.to_string())).api()?;
        ctx.json(
            rows.into_iter()
                .map(ClienteDto::from)
                .collect::<Vec<_>>(),
        )
    }

    pub async fn show(
        mut ctx: doido::controller::Context,
    ) -> doido::controller::Response {
        let access = tenant::tenant_from_ctx(ctx).await.api()?;
        let id = parse_id(&ctx, "id").api()?;
        let row = tenant_scope::cliente_for_tenant(ctx.db(), access.empresa.id, id).await.api()?;
        ctx.json(ClienteDto::from(row))
    }

    pub async fn create(
        mut ctx: doido::controller::Context,
    ) -> doido::controller::Response {
        let access = tenant::tenant_from_ctx(ctx).await.api()?;
        let form: ClienteForm = ctx
            .body_json()
            .await
            .map_err(|e| ApiError::Validation(e.to_string())).api()?;
        let record = ClienteActive {
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
        };
        let created = record
            .insert(ctx.db())
            .await
            .map_err(|e| ApiError::Internal(e.to_string())).api()?;
        ctx.json(ClienteDto::from(created))
    }

    pub async fn update(
        mut ctx: doido::controller::Context,
    ) -> doido::controller::Response {
        let access = tenant::tenant_from_ctx(ctx).await.api()?;
        let id = parse_id(&ctx, "id").api()?;
        let existing = tenant_scope::cliente_for_tenant(ctx.db(), access.empresa.id, id).await.api()?;
        let form: ClienteForm = ctx
            .body_json()
            .await
            .map_err(|e| ApiError::Validation(e.to_string())).api()?;
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
        let updated = record
            .update(ctx.db())
            .await
            .map_err(|e| ApiError::Internal(e.to_string())).api()?;
        ctx.json(ClienteDto::from(updated))
    }

    pub async fn destroy(
        mut ctx: doido::controller::Context,
    ) -> doido::controller::Response {
        let access = tenant::tenant_from_ctx(ctx).await.api()?;
        let id = parse_id(&ctx, "id").api()?;
        tenant_scope::cliente_for_tenant(ctx.db(), access.empresa.id, id).await.api()?;
        ClienteEntity::delete_by_id(id)
            .exec(ctx.db())
            .await
            .map_err(|e| ApiError::Internal(e.to_string())).api()?;
        ctx.status(204)
    }
}
