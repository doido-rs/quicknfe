use chrono::{DateTime, Utc};
use doido::controller::controller;
use doido::model::sea_orm::{ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, Set};
use serde::Deserialize;

use crate::auth::tenant;
use crate::models::log_mensagem_retorno::{
    ActiveModel as LogActive, Entity as LogEntity,
};
use crate::services::api_error::{parse_id, ApiErrExt, ApiError};
use crate::services::dto::LogRetornoDto;
use crate::services::tenant_scope;

#[derive(Debug, Deserialize)]
pub struct LogForm {
    pub documento_fiscal_id: Option<i64>,
    pub lote_id: Option<i64>,
    pub codigo_erro: String,
    pub mensagem_retorno: String,
    pub correcao_sugerida: Option<String>,
    pub data_registro: DateTime<Utc>,
}

async fn validate_log_refs(
    db: &doido::model::sea_orm::DatabaseConnection,
    empresa_id: i64,
    form: &LogForm,
) -> Result<(), ApiError> {
    match (form.documento_fiscal_id, form.lote_id) {
        (Some(doc), None) => {
            tenant_scope::documento_for_tenant(db, empresa_id, doc).await.api()?;
        }
        (None, Some(lote)) => {
            tenant_scope::lote_for_tenant(db, empresa_id, lote).await.api()?;
        }
        (Some(doc), Some(lote)) => {
            tenant_scope::documento_for_tenant(db, empresa_id, doc).await.api()?;
            tenant_scope::lote_for_tenant(db, empresa_id, lote).await.api()?;
        }
        (None, None) => return Err(ApiError::Validation(
            "documento_fiscal_id or lote_id required".into(),
        )),
    }
    Ok(())
}

pub struct ApiLogsController;

#[controller]
impl ApiLogsController {
    pub async fn index(
        mut ctx: doido::controller::Context,
    ) -> doido::controller::Response {
        let access = tenant::tenant_from_ctx(ctx).await.api()?;
        let docs = crate::models::documento_fiscal::Entity::find()
            .filter(crate::models::documento_fiscal::Column::EmpresaId.eq(access.empresa.id))
            .all(ctx.db())
            .await
            .map_err(|e| ApiError::Internal(e.to_string())).api()?;
        let doc_ids: Vec<i64> = docs.iter().map(|d| d.id).collect();
        let lotes = crate::models::lote_transmissao::Entity::find()
            .filter(crate::models::lote_transmissao::Column::EmpresaId.eq(access.empresa.id))
            .all(ctx.db())
            .await
            .map_err(|e| ApiError::Internal(e.to_string())).api()?;
        let lote_ids: Vec<i64> = lotes.iter().map(|l| l.id).collect();

        let mut condition = Condition::any();
        if !doc_ids.is_empty() {
            condition = condition.add(
                crate::models::log_mensagem_retorno::Column::DocumentoFiscalId
                    .is_in(doc_ids.clone()),
            );
        }
        if !lote_ids.is_empty() {
            condition = condition.add(
                crate::models::log_mensagem_retorno::Column::LoteId.is_in(lote_ids.clone()),
            );
        }
        if doc_ids.is_empty() && lote_ids.is_empty() {
            return ctx.json(Vec::<LogRetornoDto>::new());
        }
        let rows = LogEntity::find()
            .filter(condition)
            .all(ctx.db())
            .await
            .map_err(|e| ApiError::Internal(e.to_string())).api()?;
        ctx.json(
            rows.into_iter()
                .map(LogRetornoDto::from)
                .collect::<Vec<_>>(),
        )
    }

    pub async fn show(
        mut ctx: doido::controller::Context,
    ) -> doido::controller::Response {
        let access = tenant::tenant_from_ctx(ctx).await.api()?;
        let id = parse_id(&ctx, "id").api()?;
        let row = tenant_scope::log_for_tenant(ctx.db(), access.empresa.id, id).await.api()?;
        ctx.json(LogRetornoDto::from(row))
    }

    pub async fn update(
        mut ctx: doido::controller::Context,
    ) -> doido::controller::Response {
        let access = tenant::tenant_from_ctx(ctx).await.api()?;
        let id = parse_id(&ctx, "id").api()?;
        let existing = tenant_scope::log_for_tenant(ctx.db(), access.empresa.id, id).await.api()?;
        let form: LogForm = ctx
            .body_json()
            .await
            .map_err(|e| ApiError::Validation(e.to_string())).api()?;
        validate_log_refs(ctx.db(), access.empresa.id, &form).await.api()?;
        let mut record: LogActive = existing.into();
        record.documento_fiscal_id = Set(form.documento_fiscal_id);
        record.lote_id = Set(form.lote_id);
        record.codigo_erro = Set(form.codigo_erro);
        record.mensagem_retorno = Set(form.mensagem_retorno);
        record.correcao_sugerida = Set(form.correcao_sugerida);
        record.data_registro = Set(form.data_registro);
        let updated = record
            .update(ctx.db())
            .await
            .map_err(|e| ApiError::Internal(e.to_string())).api()?;
        ctx.json(LogRetornoDto::from(updated))
    }

    pub async fn destroy(
        mut ctx: doido::controller::Context,
    ) -> doido::controller::Response {
        let access = tenant::tenant_from_ctx(ctx).await.api()?;
        let id = parse_id(&ctx, "id").api()?;
        tenant_scope::log_for_tenant(ctx.db(), access.empresa.id, id).await.api()?;
        LogEntity::delete_by_id(id)
            .exec(ctx.db())
            .await
            .map_err(|e| ApiError::Internal(e.to_string())).api()?;
        ctx.status(204)
    }
}
