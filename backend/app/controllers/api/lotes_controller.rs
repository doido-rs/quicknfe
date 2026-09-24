use chrono::{DateTime, Utc};
use doido::controller::controller;
use doido::model::sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::Deserialize;

use crate::auth::tenant;
use crate::models::lote_transmissao::{
    ActiveModel as LoteActive, Column as LoteColumn, Entity as LoteEntity,
};
use crate::services::api_error::{parse_id, ApiErrExt, ApiError};
use crate::services::dto::LoteDto;
use crate::services::tenant_scope;

#[derive(Debug, Deserialize)]
pub struct LoteForm {
    pub numero_lote: i64,
    pub protocolo_recebimento: Option<String>,
    pub ambiente: i32,
    pub status_lote: String,
    pub data_recebimento: Option<DateTime<Utc>>,
}

pub struct ApiLotesController;

#[controller]
impl ApiLotesController {
    pub async fn index(
        mut ctx: doido::controller::Context,
    ) -> doido::controller::Response {
        let access = tenant::tenant_from_ctx(ctx).await.api()?;
        let rows = LoteEntity::find()
            .filter(LoteColumn::EmpresaId.eq(access.empresa.id))
            .all(ctx.db())
            .await
            .map_err(|e| ApiError::Internal(e.to_string())).api()?;
        ctx.json(
            rows.into_iter().map(LoteDto::from).collect::<Vec<_>>(),
        )
    }

    pub async fn show(
        mut ctx: doido::controller::Context,
    ) -> doido::controller::Response {
        let access = tenant::tenant_from_ctx(ctx).await.api()?;
        let id = parse_id(&ctx, "id").api()?;
        let row = tenant_scope::lote_for_tenant(ctx.db(), access.empresa.id, id).await.api()?;
        ctx.json(LoteDto::from(row))
    }

    pub async fn create(
        mut ctx: doido::controller::Context,
    ) -> doido::controller::Response {
        let access = tenant::tenant_from_ctx(ctx).await.api()?;
        let form: LoteForm = ctx
            .body_json()
            .await
            .map_err(|e| ApiError::Validation(e.to_string())).api()?;
        let record = LoteActive {
            empresa_id: Set(access.empresa.id),
            numero_lote: Set(form.numero_lote),
            protocolo_recebimento: Set(form.protocolo_recebimento),
            ambiente: Set(form.ambiente),
            status_lote: Set(form.status_lote),
            data_recebimento: Set(form.data_recebimento),
            ..Default::default()
        };
        let created = record
            .insert(ctx.db())
            .await
            .map_err(|e| ApiError::Internal(e.to_string())).api()?;
        ctx.json(LoteDto::from(created))
    }

    pub async fn update(
        mut ctx: doido::controller::Context,
    ) -> doido::controller::Response {
        let access = tenant::tenant_from_ctx(ctx).await.api()?;
        let id = parse_id(&ctx, "id").api()?;
        let existing = tenant_scope::lote_for_tenant(ctx.db(), access.empresa.id, id).await.api()?;
        let form: LoteForm = ctx
            .body_json()
            .await
            .map_err(|e| ApiError::Validation(e.to_string())).api()?;
        let mut record: LoteActive = existing.into();
        record.numero_lote = Set(form.numero_lote);
        record.protocolo_recebimento = Set(form.protocolo_recebimento);
        record.ambiente = Set(form.ambiente);
        record.status_lote = Set(form.status_lote);
        record.data_recebimento = Set(form.data_recebimento);
        let updated = record
            .update(ctx.db())
            .await
            .map_err(|e| ApiError::Internal(e.to_string())).api()?;
        ctx.json(LoteDto::from(updated))
    }

    pub async fn destroy(
        mut ctx: doido::controller::Context,
    ) -> doido::controller::Response {
        let access = tenant::tenant_from_ctx(ctx).await.api()?;
        let id = parse_id(&ctx, "id").api()?;
        tenant_scope::lote_for_tenant(ctx.db(), access.empresa.id, id).await.api()?;
        LoteEntity::delete_by_id(id)
            .exec(ctx.db())
            .await
            .map_err(|e| ApiError::Internal(e.to_string())).api()?;
        ctx.status(204)
    }
}
