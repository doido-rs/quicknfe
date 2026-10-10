use chrono::{DateTime, Utc};
use doido::controller::controller;
use doido::model::sea_orm::{ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, Set};
use serde::Deserialize;

use crate::models::lote_transmissao::{
    ActiveModel as LoteActive, Column as LoteColumn, Entity as LoteEntity,
};
use crate::services::api_error::{db, id_param, json_body, require_tenant, scoped, to_response};
use crate::services::pagination::{page_params_from_ctx, paginate_select};
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
        let params = crate::api!(to_response(page_params_from_ctx(&ctx)));
        let mut select = LoteEntity::find().filter(LoteColumn::EmpresaId.eq(access.prestador.id));
        if let Some(ref q) = params.q {
            let mut filter = Condition::any().add(LoteColumn::StatusLote.contains(q));
            if let Ok(num) = q.parse::<i64>() {
                filter = filter.add(LoteColumn::NumeroLote.eq(num));
            }
            select = select.filter(filter);
        }
        let page = crate::api!(db(
            paginate_select(
                ctx.db(),
                select,
                LoteColumn::Id,
                params,
                |m| m.id,
                LoteDto::from,
            )
            .await,
        ));
        ctx.json(page)
    }

    pub async fn show(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let id = crate::api!(id_param(&ctx, "id"));
        let row = crate::api!(scoped(tenant_scope::lote_for_tenant(ctx.db(), access.prestador.id, id).await));
        ctx.json(LoteDto::from(row))
    }

    pub async fn create(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let form: LoteForm = crate::api!(json_body(ctx).await);
        let created = crate::api!(db(
            LoteActive {
                empresa_id: Set(access.prestador.id),
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
        let existing = crate::api!(scoped(tenant_scope::lote_for_tenant(ctx.db(), access.prestador.id, id).await));
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
        crate::api!(scoped(tenant_scope::lote_for_tenant(ctx.db(), access.prestador.id, id).await));
        crate::api!(db(LoteEntity::delete_by_id(id).exec(ctx.db()).await));
        ctx.status(204)
    }
}
