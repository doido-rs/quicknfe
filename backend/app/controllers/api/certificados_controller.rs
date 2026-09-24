use chrono::{DateTime, Utc};
use doido::controller::controller;
use doido::model::sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::Deserialize;

use crate::models::certificado_digital::{
    ActiveModel as CertActive, Column as CertColumn, Entity as CertEntity,
};
use crate::services::api_error::{db, id_param, json_body, require_tenant, to_response, ApiError};
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
    pub async fn index(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let rows = crate::api!(db(
            CertEntity::find()
                .filter(CertColumn::EmpresaId.eq(access.empresa.id))
                .all(ctx.db())
                .await,
        ));
        ctx.json(rows.into_iter().map(CertificadoDto::from).collect::<Vec<_>>())
    }

    pub async fn show(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let id = crate::api!(id_param(&ctx, "id"));
        let row = crate::api!(db(tenant_scope::certificado_for_tenant(ctx.db(), access.empresa.id, id).await));
        ctx.json(CertificadoDto::from(row))
    }

    pub async fn create(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let form: CertificadoForm = crate::api!(json_body(ctx).await);
        let senha = crate::api!(to_response(
            form.senha
                .filter(|s| !s.is_empty())
                .ok_or(ApiError::Validation("senha is required".into())),
        ));
        let created = crate::api!(db(
            CertActive {
                empresa_id: Set(access.empresa.id),
                tipo: Set(form.tipo),
                caminho_arquivo: Set(form.caminho_arquivo),
                senha_encriptada: Set(encode_certificado_senha(&senha)),
                data_validade: Set(form.data_validade),
                criado_em: Set(Utc::now()),
                ..Default::default()
            }
            .insert(ctx.db())
            .await,
        ));
        ctx.json(CertificadoDto::from(created))
    }

    pub async fn update(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let id = crate::api!(id_param(&ctx, "id"));
        let existing = crate::api!(db(tenant_scope::certificado_for_tenant(ctx.db(), access.empresa.id, id).await));
        let form: CertificadoForm = crate::api!(json_body(ctx).await);
        let mut record: CertActive = existing.into();
        record.tipo = Set(form.tipo);
        record.caminho_arquivo = Set(form.caminho_arquivo);
        record.data_validade = Set(form.data_validade);
        if let Some(senha) = form.senha.filter(|s| !s.is_empty()) {
            record.senha_encriptada = Set(encode_certificado_senha(&senha));
        }
        let updated = crate::api!(db(record.update(ctx.db()).await));
        ctx.json(CertificadoDto::from(updated))
    }

    pub async fn destroy(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let id = crate::api!(id_param(&ctx, "id"));
        crate::api!(db(tenant_scope::certificado_for_tenant(ctx.db(), access.empresa.id, id).await));
        crate::api!(db(CertEntity::delete_by_id(id).exec(ctx.db()).await));
        ctx.status(204)
    }
}
