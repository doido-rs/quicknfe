use doido::controller::controller;
use doido::model::sea_orm::prelude::Decimal;
use doido::model::sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::Deserialize;

use crate::auth::tenant;
use crate::models::item_documento_fiscal::{
    ActiveModel as ItemActive, Column as ItemColumn, Entity as ItemEntity,
};
use crate::services::api_error::{parse_id, ApiErrExt, ApiError};
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
    pub async fn index(
        mut ctx: doido::controller::Context,
    ) -> doido::controller::Response {
        let access = tenant::tenant_from_ctx(ctx).await.api()?;
        let doc_id = parse_id(&ctx, "doc_id").api()?;
        tenant_scope::documento_for_tenant(ctx.db(), access.empresa.id, doc_id).await.api()?;
        let rows = ItemEntity::find()
            .filter(ItemColumn::DocumentoFiscalId.eq(doc_id))
            .all(ctx.db())
            .await
            .map_err(|e| ApiError::Internal(e.to_string())).api()?;
        ctx.json(
            rows.into_iter().map(ItemDto::from).collect::<Vec<_>>(),
        )
    }

    pub async fn show(
        mut ctx: doido::controller::Context,
    ) -> doido::controller::Response {
        let access = tenant::tenant_from_ctx(ctx).await.api()?;
        let doc_id = parse_id(&ctx, "doc_id").api()?;
        let id = parse_id(&ctx, "id").api()?;
        let row =
            tenant_scope::item_for_tenant_and_doc(ctx.db(), access.empresa.id, doc_id, id).await.api()?;
        ctx.json(ItemDto::from(row))
    }

    pub async fn create(
        mut ctx: doido::controller::Context,
    ) -> doido::controller::Response {
        let access = tenant::tenant_from_ctx(ctx).await.api()?;
        let doc_id = parse_id(&ctx, "doc_id").api()?;
        tenant_scope::documento_for_tenant(ctx.db(), access.empresa.id, doc_id).await.api()?;
        let form: ItemForm = ctx
            .body_json()
            .await
            .map_err(|e| ApiError::Validation(e.to_string())).api()?;
        let record = ItemActive {
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
        };
        let created = record
            .insert(ctx.db())
            .await
            .map_err(|e| ApiError::Internal(e.to_string())).api()?;
        ctx.json(ItemDto::from(created))
    }

    pub async fn update(
        mut ctx: doido::controller::Context,
    ) -> doido::controller::Response {
        let access = tenant::tenant_from_ctx(ctx).await.api()?;
        let doc_id = parse_id(&ctx, "doc_id").api()?;
        let id = parse_id(&ctx, "id").api()?;
        let existing =
            tenant_scope::item_for_tenant_and_doc(ctx.db(), access.empresa.id, doc_id, id).await.api()?;
        let form: ItemForm = ctx
            .body_json()
            .await
            .map_err(|e| ApiError::Validation(e.to_string())).api()?;
        let mut record: ItemActive = existing.into();
        record.numero_item = Set(form.numero_item);
        record.descricao = Set(form.descricao);
        record.codigo_ncm = Set(form.codigo_ncm);
        record.codigo_tributacao_nacional = Set(form.codigo_tributacao_nacional);
        record.quantidade = Set(form.quantidade);
        record.valor_unitario = Set(form.valor_unitario);
        record.valor_total_bruto = Set(form.valor_total_bruto);
        record.valor_desconto = Set(form.valor_desconto);
        let updated = record
            .update(ctx.db())
            .await
            .map_err(|e| ApiError::Internal(e.to_string())).api()?;
        ctx.json(ItemDto::from(updated))
    }

    pub async fn destroy(
        mut ctx: doido::controller::Context,
    ) -> doido::controller::Response {
        let access = tenant::tenant_from_ctx(ctx).await.api()?;
        let doc_id = parse_id(&ctx, "doc_id").api()?;
        let id = parse_id(&ctx, "id").api()?;
        tenant_scope::item_for_tenant_and_doc(ctx.db(), access.empresa.id, doc_id, id).await.api()?;
        ItemEntity::delete_by_id(id)
            .exec(ctx.db())
            .await
            .map_err(|e| ApiError::Internal(e.to_string())).api()?;
        ctx.status(204)
    }
}
