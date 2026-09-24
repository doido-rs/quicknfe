use chrono::{DateTime, Utc};
use doido::controller::controller;
use doido::model::sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::Deserialize;

use crate::auth::tenant;
use crate::models::evento_documento_fiscal::{
    ActiveModel as EventoActive, Column as EventoColumn, Entity as EventoEntity,
};
use crate::services::api_error::{parse_id, ApiErrExt, ApiError};
use crate::services::dto::EventoDto;
use crate::services::tenant_scope;

#[derive(Debug, Deserialize)]
pub struct EventoForm {
    pub tipo_evento: String,
    pub sequencial_evento: i32,
    pub protocolo_evento: Option<String>,
    pub descricao_motivo: Option<String>,
    pub data_hora_evento: DateTime<Utc>,
    pub ambiente: i32,
    pub xml_evento: Option<String>,
}

pub struct ApiEventosController;

#[controller]
impl ApiEventosController {
    pub async fn index(
        mut ctx: doido::controller::Context,
    ) -> doido::controller::Response {
        let access = tenant::tenant_from_ctx(ctx).await.api()?;
        let doc_id = parse_id(&ctx, "doc_id").api()?;
        tenant_scope::documento_for_tenant(ctx.db(), access.empresa.id, doc_id).await.api()?;
        let rows = EventoEntity::find()
            .filter(EventoColumn::DocumentoFiscalId.eq(doc_id))
            .all(ctx.db())
            .await
            .map_err(|e| ApiError::Internal(e.to_string())).api()?;
        ctx.json(
            rows.into_iter().map(EventoDto::from).collect::<Vec<_>>(),
        )
    }

    pub async fn show(
        mut ctx: doido::controller::Context,
    ) -> doido::controller::Response {
        let access = tenant::tenant_from_ctx(ctx).await.api()?;
        let doc_id = parse_id(&ctx, "doc_id").api()?;
        let id = parse_id(&ctx, "id").api()?;
        let row =
            tenant_scope::evento_for_tenant(ctx.db(), access.empresa.id, doc_id, id).await.api()?;
        ctx.json(EventoDto::from(row))
    }

    pub async fn create(
        mut ctx: doido::controller::Context,
    ) -> doido::controller::Response {
        let access = tenant::tenant_from_ctx(ctx).await.api()?;
        let doc_id = parse_id(&ctx, "doc_id").api()?;
        tenant_scope::documento_for_tenant(ctx.db(), access.empresa.id, doc_id).await.api()?;
        let form: EventoForm = ctx
            .body_json()
            .await
            .map_err(|e| ApiError::Validation(e.to_string())).api()?;
        let record = EventoActive {
            documento_fiscal_id: Set(doc_id),
            tipo_evento: Set(form.tipo_evento),
            sequencial_evento: Set(form.sequencial_evento),
            protocolo_evento: Set(form.protocolo_evento),
            descricao_motivo: Set(form.descricao_motivo),
            data_hora_evento: Set(form.data_hora_evento),
            ambiente: Set(form.ambiente),
            xml_evento: Set(form.xml_evento),
            ..Default::default()
        };
        let created = record
            .insert(ctx.db())
            .await
            .map_err(|e| ApiError::Internal(e.to_string())).api()?;
        ctx.json(EventoDto::from(created))
    }

    pub async fn update(
        mut ctx: doido::controller::Context,
    ) -> doido::controller::Response {
        let access = tenant::tenant_from_ctx(ctx).await.api()?;
        let doc_id = parse_id(&ctx, "doc_id").api()?;
        let id = parse_id(&ctx, "id").api()?;
        let existing =
            tenant_scope::evento_for_tenant(ctx.db(), access.empresa.id, doc_id, id).await.api()?;
        let form: EventoForm = ctx
            .body_json()
            .await
            .map_err(|e| ApiError::Validation(e.to_string())).api()?;
        let mut record: EventoActive = existing.into();
        record.tipo_evento = Set(form.tipo_evento);
        record.sequencial_evento = Set(form.sequencial_evento);
        record.protocolo_evento = Set(form.protocolo_evento);
        record.descricao_motivo = Set(form.descricao_motivo);
        record.data_hora_evento = Set(form.data_hora_evento);
        record.ambiente = Set(form.ambiente);
        record.xml_evento = Set(form.xml_evento);
        let updated = record
            .update(ctx.db())
            .await
            .map_err(|e| ApiError::Internal(e.to_string())).api()?;
        ctx.json(EventoDto::from(updated))
    }

    pub async fn destroy(
        mut ctx: doido::controller::Context,
    ) -> doido::controller::Response {
        let access = tenant::tenant_from_ctx(ctx).await.api()?;
        let doc_id = parse_id(&ctx, "doc_id").api()?;
        let id = parse_id(&ctx, "id").api()?;
        tenant_scope::evento_for_tenant(ctx.db(), access.empresa.id, doc_id, id).await.api()?;
        EventoEntity::delete_by_id(id)
            .exec(ctx.db())
            .await
            .map_err(|e| ApiError::Internal(e.to_string())).api()?;
        ctx.status(204)
    }
}
