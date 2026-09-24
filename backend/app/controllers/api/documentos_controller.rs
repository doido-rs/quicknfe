use chrono::{DateTime, NaiveDate, Utc};
use doido::controller::controller;
use doido::model::sea_orm::prelude::Decimal;
use doido::model::sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::Deserialize;

use crate::auth::tenant;
use crate::models::documento_fiscal::{
    ActiveModel as DocActive, Column as DocColumn, Entity as DocEntity,
};
use crate::services::api_error::{parse_id, ApiErrExt, ApiError};
use crate::services::dto::{DocumentoDto, DocumentoListDto};
use crate::services::tenant_scope;

#[derive(Debug, Deserialize)]
pub struct DocumentoForm {
    pub cliente_id: Option<i64>,
    pub lote_id: Option<i64>,
    pub modelo: String,
    pub serie: String,
    pub numero_documento: i64,
    pub chave_acesso: Option<String>,
    pub codigo_verificacao: Option<String>,
    pub ambiente: i32,
    pub tipo_emissao: i32,
    pub status: String,
    pub data_emissao: DateTime<Utc>,
    pub data_competencia: Option<NaiveDate>,
    pub codigo_municipio_emissao: String,
    pub protocolo_autorizacao: Option<String>,
    pub valor_total_servicos_produtos: Decimal,
    pub valor_liquido: Decimal,
    pub xml_assinado: Option<String>,
    pub xml_processado: Option<String>,
}

async fn validate_refs(
    db: &doido::model::sea_orm::DatabaseConnection,
    empresa_id: i64,
    form: &DocumentoForm,
) -> Result<(), ApiError> {
    if let Some(cid) = form.cliente_id {
        tenant_scope::ensure_cliente_belongs(db, empresa_id, cid).await.api()?;
    }
    if let Some(lid) = form.lote_id {
        tenant_scope::ensure_lote_belongs(db, empresa_id, lid).await.api()?;
    }
    Ok(())
}

fn apply_form(record: &mut DocActive, form: &DocumentoForm, empresa_id: i64) {
    record.empresa_id = Set(empresa_id);
    record.cliente_id = Set(form.cliente_id);
    record.lote_id = Set(form.lote_id);
    record.modelo = Set(form.modelo.clone());
    record.serie = Set(form.serie.clone());
    record.numero_documento = Set(form.numero_documento);
    record.chave_acesso = Set(form.chave_acesso.clone());
    record.codigo_verificacao = Set(form.codigo_verificacao.clone());
    record.ambiente = Set(form.ambiente);
    record.tipo_emissao = Set(form.tipo_emissao);
    record.status = Set(form.status.clone());
    record.data_emissao = Set(form.data_emissao);
    record.data_competencia = Set(form.data_competencia);
    record.codigo_municipio_emissao = Set(form.codigo_municipio_emissao.clone());
    record.protocolo_autorizacao = Set(form.protocolo_autorizacao.clone());
    record.valor_total_servicos_produtos = Set(form.valor_total_servicos_produtos);
    record.valor_liquido = Set(form.valor_liquido);
    record.xml_assinado = Set(form.xml_assinado.clone());
    record.xml_processado = Set(form.xml_processado.clone());
}

pub struct ApiDocumentosController;

#[controller]
impl ApiDocumentosController {
    pub async fn index(
        mut ctx: doido::controller::Context,
    ) -> doido::controller::Response {
        let access = tenant::tenant_from_ctx(ctx).await.api()?;
        let rows = DocEntity::find()
            .filter(DocColumn::EmpresaId.eq(access.empresa.id))
            .all(ctx.db())
            .await
            .map_err(|e| ApiError::Internal(e.to_string())).api()?;
        ctx.json(
            rows.into_iter()
                .map(DocumentoListDto::from)
                .collect::<Vec<_>>(),
        )
    }

    pub async fn show(
        mut ctx: doido::controller::Context,
    ) -> doido::controller::Response {
        let access = tenant::tenant_from_ctx(ctx).await.api()?;
        let id = parse_id(&ctx, "id").api()?;
        let row = tenant_scope::documento_for_tenant(ctx.db(), access.empresa.id, id).await.api()?;
        ctx.json(DocumentoDto::from(row))
    }

    pub async fn create(
        mut ctx: doido::controller::Context,
    ) -> doido::controller::Response {
        let access = tenant::tenant_from_ctx(ctx).await.api()?;
        let form: DocumentoForm = ctx
            .body_json()
            .await
            .map_err(|e| ApiError::Validation(e.to_string())).api()?;
        validate_refs(ctx.db(), access.empresa.id, &form).await.api()?;
        let mut record = DocActive {
            ..Default::default()
        };
        apply_form(&mut record, &form, access.empresa.id);
        let created = record
            .insert(ctx.db())
            .await
            .map_err(|e| ApiError::Internal(e.to_string())).api()?;
        ctx.json(DocumentoDto::from(created))
    }

    pub async fn update(
        mut ctx: doido::controller::Context,
    ) -> doido::controller::Response {
        let access = tenant::tenant_from_ctx(ctx).await.api()?;
        let id = parse_id(&ctx, "id").api()?;
        let existing =
            tenant_scope::documento_for_tenant(ctx.db(), access.empresa.id, id).await.api()?;
        let form: DocumentoForm = ctx
            .body_json()
            .await
            .map_err(|e| ApiError::Validation(e.to_string())).api()?;
        validate_refs(ctx.db(), access.empresa.id, &form).await.api()?;
        let mut record: DocActive = existing.into();
        apply_form(&mut record, &form, access.empresa.id);
        let updated = record
            .update(ctx.db())
            .await
            .map_err(|e| ApiError::Internal(e.to_string())).api()?;
        ctx.json(DocumentoDto::from(updated))
    }

    pub async fn destroy(
        mut ctx: doido::controller::Context,
    ) -> doido::controller::Response {
        let access = tenant::tenant_from_ctx(ctx).await.api()?;
        let id = parse_id(&ctx, "id").api()?;
        tenant_scope::documento_for_tenant(ctx.db(), access.empresa.id, id).await.api()?;
        DocEntity::delete_by_id(id)
            .exec(ctx.db())
            .await
            .map_err(|e| ApiError::Internal(e.to_string())).api()?;
        ctx.status(204)
    }
}
