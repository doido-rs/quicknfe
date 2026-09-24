use chrono::{DateTime, Utc};
use doido::controller::controller;
use doido::model::sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::Deserialize;

use crate::models::lote_transmissao::{
    ActiveModel as LoteActive, Column as LoteColumn, Entity as LoteEntity,
};
use crate::services::api_error::{db, id_param, json_body, require_tenant};
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
    pub async fn index(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let rows = crate::api!(db(
            LoteEntity::find()
                .filter(LoteColumn::EmpresaId.eq(access.empresa.id))
                .all(ctx.db())
                .await,
        ));
        ctx.json(rows.into_iter().map(LoteDto::from).collect::<Vec<_>>())
    }

    pub async fn show(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let id = crate::api!(id_param(&ctx, "id"));
        let row = crate::api!(db(tenant_scope::lote_for_tenant(ctx.db(), access.empresa.id, id).await));
        ctx.json(LoteDto::from(row))
    }

    pub async fn create(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let form: LoteForm = crate::api!(json_body(ctx).await);
        let created = crate::api!(db(
            LoteActive {
                empresa_id: Set(access.empresa.id),
                numero_lote: Set(form.numero_lote),
                protocolo_recebimento: Set(form.protocolo_recebimento),
                ambiente: Set(form.ambiente),
                status_lote: Set(form.status_lote),
                data_recebimento: Set(form.data_recebimento),
                ..Default::default()
            }
            .insert(ctx.db())
            .await,
        ));
        ctx.json(LoteDto::from(created))
    }

    pub async fn update(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let id = crate::api!(id_param(&ctx, "id"));
        let existing = crate::api!(db(tenant_scope::lote_for_tenant(ctx.db(), access.empresa.id, id).await));
        let form: LoteForm = crate::api!(json_body(ctx).await);
        let mut record: LoteActive = existing.into();
        record.numero_lote = Set(form.numero_lote);
        record.protocolo_recebimento = Set(form.protocolo_recebimento);
        record.ambiente = Set(form.ambiente);
        record.status_lote = Set(form.status_lote);
        record.data_recebimento = Set(form.data_recebimento);
        let updated = crate::api!(db(record.update(ctx.db()).await));
        ctx.json(LoteDto::from(updated))
    }

    pub async fn destroy(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let id = crate::api!(id_param(&ctx, "id"));
        crate::api!(db(tenant_scope::lote_for_tenant(ctx.db(), access.empresa.id, id).await));
        crate::api!(db(LoteEntity::delete_by_id(id).exec(ctx.db()).await));
        ctx.status(204)
    }
}
