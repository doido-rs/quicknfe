use chrono::{DateTime, Utc};
use doido::controller::controller;
use doido::model::sea_orm::{ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, Set};
use crate::models::log_mensagem_retorno::Column as LogColumn;
use serde::Deserialize;

use crate::models::log_mensagem_retorno::{ActiveModel as LogActive, Entity as LogEntity};
use crate::services::api_error::{db, id_param, json_body, require_tenant, scoped, to_response, ApiError};
use crate::services::dto::LogRetornoDto;
use crate::services::pagination::{page_params_from_ctx, paginate_select, PageResponse};
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
            tenant_scope::documento_for_tenant(db, empresa_id, doc).await?;
        }
        (None, Some(lote)) => {
            tenant_scope::lote_for_tenant(db, empresa_id, lote).await?;
        }
        (Some(doc), Some(lote)) => {
            tenant_scope::documento_for_tenant(db, empresa_id, doc).await?;
            tenant_scope::lote_for_tenant(db, empresa_id, lote).await?;
        }
        (None, None) => {
            return Err(ApiError::Validation(
                "documento_fiscal_id or lote_id required".into(),
            ))
        }
    }
    Ok(())
}

pub struct ApiLogsController;

#[controller]
impl ApiLogsController {
    pub async fn index(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let params = crate::api!(to_response(page_params_from_ctx(&ctx)));
        let docs = crate::api!(db(
            crate::models::documento_fiscal::Entity::find()
                .filter(crate::models::documento_fiscal::Column::EmpresaId.eq(access.prestador.id))
                .all(ctx.db())
                .await,
        ));
        let doc_ids: Vec<i64> = docs.iter().map(|d| d.id).collect();
        let lotes = crate::api!(db(
            crate::models::lote_transmissao::Entity::find()
                .filter(crate::models::lote_transmissao::Column::EmpresaId.eq(access.prestador.id))
                .all(ctx.db())
                .await,
        ));
        let lote_ids: Vec<i64> = lotes.iter().map(|l| l.id).collect();

        if doc_ids.is_empty() && lote_ids.is_empty() {
            return ctx.json(PageResponse {
                items: Vec::<LogRetornoDto>::new(),
                next_page_token: None,
            });
        }

        let mut condition = Condition::any();
        if !doc_ids.is_empty() {
            condition = condition.add(LogColumn::DocumentoFiscalId.is_in(doc_ids));
        }
        if !lote_ids.is_empty() {
            condition = condition.add(LogColumn::LoteId.is_in(lote_ids));
        }

        let mut select = LogEntity::find().filter(condition);
        if let Some(ref q) = params.q {
            select = select.filter(
                Condition::any()
                    .add(LogColumn::CodigoErro.contains(q))
                    .add(LogColumn::MensagemRetorno.contains(q)),
            );
        }
        let page = crate::api!(db(
            paginate_select(
                ctx.db(),
                select,
                LogColumn::Id,
                params,
                |m| m.id,
                LogRetornoDto::from,
            )
            .await,
        ));
        ctx.json(page)
    }

    pub async fn index_for_documento(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let doc_id = crate::api!(id_param(&ctx, "doc_id"));
        crate::api!(scoped(
            tenant_scope::documento_for_company(ctx.db(), access.company.id, doc_id).await,
        ));
        let params = crate::api!(to_response(page_params_from_ctx(&ctx)));
        let mut select = LogEntity::find().filter(LogColumn::DocumentoFiscalId.eq(doc_id));
        if let Some(ref q) = params.q {
            select = select.filter(
                Condition::any()
                    .add(LogColumn::CodigoErro.contains(q))
                    .add(LogColumn::MensagemRetorno.contains(q)),
            );
        }
        let page = crate::api!(db(
            paginate_select(
                ctx.db(),
                select,
                LogColumn::Id,
                params,
                |m| m.id,
                LogRetornoDto::from,
            )
            .await,
        ));
        ctx.json(page)
    }

    pub async fn show_for_documento(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let doc_id = crate::api!(id_param(&ctx, "doc_id"));
        let id = crate::api!(id_param(&ctx, "id"));
        let doc = crate::api!(scoped(
            tenant_scope::documento_for_company(ctx.db(), access.company.id, doc_id).await,
        ));
        let row = crate::api!(db(
            tenant_scope::log_for_tenant_and_doc(ctx.db(), doc.empresa_id, doc_id, id).await,
        ));
        ctx.json(LogRetornoDto::from(row))
    }

    pub async fn show(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let id = crate::api!(id_param(&ctx, "id"));
        let row = crate::api!(scoped(tenant_scope::log_for_tenant(ctx.db(), access.prestador.id, id).await));
        ctx.json(LogRetornoDto::from(row))
    }

    pub async fn update(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let id = crate::api!(id_param(&ctx, "id"));
        let existing = crate::api!(scoped(tenant_scope::log_for_tenant(ctx.db(), access.prestador.id, id).await));
        let form: LogForm = crate::api!(json_body(ctx).await);
        crate::api!(to_response(validate_log_refs(ctx.db(), access.prestador.id, &form).await));
        let mut record: LogActive = existing.into();
        record.documento_fiscal_id = Set(form.documento_fiscal_id);
        record.lote_id = Set(form.lote_id);
        record.codigo_erro = Set(form.codigo_erro);
        record.mensagem_retorno = Set(form.mensagem_retorno);
        record.correcao_sugerida = Set(form.correcao_sugerida);
        record.data_registro = Set(form.data_registro);
        let updated = crate::api!(db(record.update(ctx.db()).await));
        ctx.json(LogRetornoDto::from(updated))
    }

    pub async fn destroy(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let id = crate::api!(id_param(&ctx, "id"));
        crate::api!(scoped(tenant_scope::log_for_tenant(ctx.db(), access.prestador.id, id).await));
        crate::api!(db(LogEntity::delete_by_id(id).exec(ctx.db()).await));
        ctx.status(204)
    }
}
