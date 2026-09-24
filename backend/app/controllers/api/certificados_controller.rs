use chrono::{DateTime, Utc};
use doido::controller::controller;
use doido::model::sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::Deserialize;

use crate::auth::tenant;
use crate::models::certificado_digital::{
    ActiveModel as CertActive, Column as CertColumn, Entity as CertEntity,
};
use crate::services::api_error::{parse_id, ApiErrExt, ApiError};
use crate::services::dto::{encode_certificado_senha, CertificadoDto};
use crate::services::tenant_scope;

#[derive(Debug, Deserialize)]
pub struct CertificadoForm {
    pub tipo: String,
    pub caminho_arquivo: Option<String>,
    pub senha: Option<String>,
    pub data_validade: DateTime<Utc>,
}

pub struct ApiCertificadosController;

#[controller]
impl ApiCertificadosController {
    pub async fn index(
        mut ctx: doido::controller::Context,
    ) -> doido::controller::Response {
        let access = tenant::tenant_from_ctx(ctx).await.api()?;
        let rows = CertEntity::find()
            .filter(CertColumn::EmpresaId.eq(access.empresa.id))
            .all(ctx.db())
            .await
            .map_err(|e| ApiError::Internal(e.to_string())).api()?;
        ctx.json(
            rows.into_iter()
                .map(CertificadoDto::from)
                .collect::<Vec<_>>(),
        )
    }

    pub async fn show(
        mut ctx: doido::controller::Context,
    ) -> doido::controller::Response {
        let access = tenant::tenant_from_ctx(ctx).await.api()?;
        let id = parse_id(&ctx, "id").api()?;
        let row = tenant_scope::certificado_for_tenant(ctx.db(), access.empresa.id, id).await.api()?;
        ctx.json(CertificadoDto::from(row))
    }

    pub async fn create(
        mut ctx: doido::controller::Context,
    ) -> doido::controller::Response {
        let access = tenant::tenant_from_ctx(ctx).await.api()?;
        let form: CertificadoForm = ctx
            .body_json()
            .await
            .map_err(|e| ApiError::Validation(e.to_string())).api()?;
        let senha = form
            .senha
            .filter(|s| !s.is_empty())
            .ok_or_else(|| ApiError::Validation("senha is required".into())).api()?;
        let now = Utc::now();
        let record = CertActive {
            empresa_id: Set(access.empresa.id),
            tipo: Set(form.tipo),
            caminho_arquivo: Set(form.caminho_arquivo),
            senha_encriptada: Set(encode_certificado_senha(&senha)),
            data_validade: Set(form.data_validade),
            criado_em: Set(now),
            ..Default::default()
        };
        let created = record
            .insert(ctx.db())
            .await
            .map_err(|e| ApiError::Internal(e.to_string())).api()?;
        ctx.json(CertificadoDto::from(created))
    }

    pub async fn update(
        mut ctx: doido::controller::Context,
    ) -> doido::controller::Response {
        let access = tenant::tenant_from_ctx(ctx).await.api()?;
        let id = parse_id(&ctx, "id").api()?;
        let existing =
            tenant_scope::certificado_for_tenant(ctx.db(), access.empresa.id, id).await.api()?;
        let form: CertificadoForm = ctx
            .body_json()
            .await
            .map_err(|e| ApiError::Validation(e.to_string())).api()?;
        let mut record: CertActive = existing.into();
        record.tipo = Set(form.tipo);
        record.caminho_arquivo = Set(form.caminho_arquivo);
        record.data_validade = Set(form.data_validade);
        if let Some(senha) = form.senha.filter(|s| !s.is_empty()) {
            record.senha_encriptada = Set(encode_certificado_senha(&senha));
        }
        let updated = record
            .update(ctx.db())
            .await
            .map_err(|e| ApiError::Internal(e.to_string())).api()?;
        ctx.json(CertificadoDto::from(updated))
    }

    pub async fn destroy(
        mut ctx: doido::controller::Context,
    ) -> doido::controller::Response {
        let access = tenant::tenant_from_ctx(ctx).await.api()?;
        let id = parse_id(&ctx, "id").api()?;
        tenant_scope::certificado_for_tenant(ctx.db(), access.empresa.id, id).await.api()?;
        CertEntity::delete_by_id(id)
            .exec(ctx.db())
            .await
            .map_err(|e| ApiError::Internal(e.to_string())).api()?;
        ctx.status(204)
    }
}
