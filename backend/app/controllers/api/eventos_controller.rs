use chrono::{DateTime, Utc};
use doido::controller::controller;
use doido::model::sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::Deserialize;

use crate::models::evento_documento_fiscal::{
    ActiveModel as EventoActive, Column as EventoColumn, Entity as EventoEntity,
};
use crate::services::api_error::{db, id_param, json_body, require_tenant, scoped, to_response};
use crate::services::pagination::{page_params_from_ctx, paginate_select};
use crate::models::tipo_evento::TipoEvento;
use crate::services::dto::EventoDto;
use crate::services::tenant_scope;

#[derive(Debug, Deserialize)]
pub struct EventoForm {
    pub tipo_evento: TipoEvento,
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
    pub async fn index(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let doc_id = crate::api!(id_param(&ctx, "doc_id"));
        crate::api!(scoped(
            tenant_scope::documento_for_company(ctx.db(), access.company.id, doc_id).await,
        ));
        let params = crate::api!(to_response(page_params_from_ctx(&ctx)));
        let mut select = EventoEntity::find().filter(EventoColumn::DocumentoFiscalId.eq(doc_id));
        if let Some(ref q) = params.q {
            select = select.filter(EventoColumn::TipoEvento.contains(q));
        }
        let page = crate::api!(db(
            paginate_select(
                ctx.db(),
                select,
                EventoColumn::Id,
                params,
                |m| m.id,
                EventoDto::from,
            )
            .await,
        ));
        ctx.json(page)
    }

    pub async fn show(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let doc_id = crate::api!(id_param(&ctx, "doc_id"));
        let id = crate::api!(id_param(&ctx, "id"));
        let doc = crate::api!(scoped(
            tenant_scope::documento_for_company(ctx.db(), access.company.id, doc_id).await,
        ));
        let row = crate::api!(db(
            tenant_scope::evento_for_tenant(ctx.db(), doc.empresa_id, doc_id, id).await,
        ));
        ctx.json(EventoDto::from(row))
    }

    pub async fn create(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let doc_id = crate::api!(id_param(&ctx, "doc_id"));
        crate::api!(scoped(
            tenant_scope::documento_for_company(ctx.db(), access.company.id, doc_id).await,
        ));
        let form: EventoForm = crate::api!(json_body(ctx).await);
        let created = crate::api!(db(
            EventoActive {
                documento_fiscal_id: Set(doc_id),
                tipo_evento: Set(form.tipo_evento.as_db_code()),
                sequencial_evento: Set(form.sequencial_evento),
                protocolo_evento: Set(form.protocolo_evento),
                descricao_motivo: Set(form.descricao_motivo),
                data_hora_evento: Set(form.data_hora_evento),
                ambiente: Set(form.ambiente),
                xml_evento: Set(form.xml_evento),
                ..Default::default()
            }
            .insert(ctx.db())
            .await,
        ));
        ctx.json(EventoDto::from(created))
    }

    pub async fn update(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let doc_id = crate::api!(id_param(&ctx, "doc_id"));
        let id = crate::api!(id_param(&ctx, "id"));
        let doc = crate::api!(scoped(
            tenant_scope::documento_for_company(ctx.db(), access.company.id, doc_id).await,
        ));
        let existing = crate::api!(db(
            tenant_scope::evento_for_tenant(ctx.db(), doc.empresa_id, doc_id, id).await,
        ));
        let form: EventoForm = crate::api!(json_body(ctx).await);
        let mut record: EventoActive = existing.into();
        record.tipo_evento = Set(form.tipo_evento.as_db_code());
        record.sequencial_evento = Set(form.sequencial_evento);
        record.protocolo_evento = Set(form.protocolo_evento);
        record.descricao_motivo = Set(form.descricao_motivo);
        record.data_hora_evento = Set(form.data_hora_evento);
        record.ambiente = Set(form.ambiente);
        record.xml_evento = Set(form.xml_evento);
        let updated = crate::api!(db(record.update(ctx.db()).await));
        ctx.json(EventoDto::from(updated))
    }

    pub async fn destroy(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let doc_id = crate::api!(id_param(&ctx, "doc_id"));
        let id = crate::api!(id_param(&ctx, "id"));
        let doc = crate::api!(scoped(
            tenant_scope::documento_for_company(ctx.db(), access.company.id, doc_id).await,
        ));
        crate::api!(db(
            tenant_scope::evento_for_tenant(ctx.db(), doc.empresa_id, doc_id, id).await,
        ));
        crate::api!(db(EventoEntity::delete_by_id(id).exec(ctx.db()).await));
        ctx.status(204)
    }
}
