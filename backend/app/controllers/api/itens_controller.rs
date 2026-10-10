use doido::controller::controller;
use doido::model::sea_orm::prelude::Decimal;
use doido::model::sea_orm::{ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, Set};
use serde::Deserialize;

use crate::models::item_documento_fiscal::{
    ActiveModel as ItemActive, Column as ItemColumn, Entity as ItemEntity,
};
use crate::services::api_error::{db, id_param, json_body, require_tenant, scoped, to_response};
use crate::services::pagination::{page_params_from_ctx, paginate_select};
use crate::services::dto::ItemDto;
use crate::services::tenant_scope;

#[derive(Debug, Deserialize)]
pub struct ItemForm {
    pub numero_item: i32,
    pub descricao: String,
    pub codigo_ncm: Option<String>,
    pub codigo_tributacao_nacional: Option<String>,
    pub quantidade: Decimal,
    pub valor_unitario: Decimal,
    pub valor_total_bruto: Decimal,
    pub valor_desconto: Option<Decimal>,
}

pub struct ApiItensController;

#[controller]
impl ApiItensController {
    pub async fn index(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let doc_id = crate::api!(id_param(&ctx, "doc_id"));
        crate::api!(scoped(tenant_scope::documento_for_tenant(ctx.db(), access.prestador.id, doc_id).await));
        let params = crate::api!(to_response(page_params_from_ctx(&ctx)));
        let mut select = ItemEntity::find().filter(ItemColumn::DocumentoFiscalId.eq(doc_id));
        if let Some(ref q) = params.q {
            select = select.filter(ItemColumn::Descricao.contains(q));
        }
        let page = crate::api!(db(
            paginate_select(
                ctx.db(),
                select,
                ItemColumn::Id,
                params,
                |m| m.id,
                ItemDto::from,
            )
            .await,
        ));
        ctx.json(page)
    }

    pub async fn show(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let doc_id = crate::api!(id_param(&ctx, "doc_id"));
        let id = crate::api!(id_param(&ctx, "id"));
        let row = crate::api!(db(
            tenant_scope::item_for_tenant_and_doc(ctx.db(), access.prestador.id, doc_id, id).await,
        ));
        ctx.json(ItemDto::from(row))
    }

    pub async fn create(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let doc_id = crate::api!(id_param(&ctx, "doc_id"));
        crate::api!(scoped(tenant_scope::documento_for_tenant(ctx.db(), access.prestador.id, doc_id).await));
        let form: ItemForm = crate::api!(json_body(ctx).await);
        let created = crate::api!(db(
            ItemActive {
                documento_fiscal_id: Set(doc_id),
                numero_item: Set(form.numero_item),
                descricao: Set(form.descricao),
                codigo_ncm: Set(form.codigo_ncm),
                codigo_tributacao_nacional: Set(form.codigo_tributacao_nacional),
                quantidade: Set(form.quantidade),
                valor_unitario: Set(form.valor_unitario),
                valor_total_bruto: Set(form.valor_total_bruto),
                valor_desconto: Set(form.valor_desconto),
                ..Default::default()
            }
            .insert(ctx.db())
            .await,
        ));
        ctx.json(ItemDto::from(created))
    }

    pub async fn update(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let doc_id = crate::api!(id_param(&ctx, "doc_id"));
        let id = crate::api!(id_param(&ctx, "id"));
        let existing = crate::api!(db(
            tenant_scope::item_for_tenant_and_doc(ctx.db(), access.prestador.id, doc_id, id).await,
        ));
        let form: ItemForm = crate::api!(json_body(ctx).await);
        let mut record: ItemActive = existing.into();
        record.numero_item = Set(form.numero_item);
        record.descricao = Set(form.descricao);
        record.codigo_ncm = Set(form.codigo_ncm);
        record.codigo_tributacao_nacional = Set(form.codigo_tributacao_nacional);
        record.quantidade = Set(form.quantidade);
        record.valor_unitario = Set(form.valor_unitario);
        record.valor_total_bruto = Set(form.valor_total_bruto);
        record.valor_desconto = Set(form.valor_desconto);
        let updated = crate::api!(db(record.update(ctx.db()).await));
        ctx.json(ItemDto::from(updated))
    }

    pub async fn destroy(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let doc_id = crate::api!(id_param(&ctx, "doc_id"));
        let id = crate::api!(id_param(&ctx, "id"));
        crate::api!(db(
            tenant_scope::item_for_tenant_and_doc(ctx.db(), access.prestador.id, doc_id, id).await,
        ));
        crate::api!(db(ItemEntity::delete_by_id(id).exec(ctx.db()).await));
        ctx.status(204)
    }
}
