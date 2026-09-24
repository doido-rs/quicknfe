use doido::controller::controller;
use doido::model::sea_orm::prelude::Decimal;
use doido::model::sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::Deserialize;

use crate::auth::tenant;
use crate::models::tributo_item::{
    ActiveModel as TributoActive, Column as TributoColumn, Entity as TributoEntity,
};
use crate::services::api_error::{parse_id, ApiErrExt, ApiError};
use crate::services::dto::TributoDto;
use crate::services::tenant_scope;

#[derive(Debug, Deserialize)]
pub struct TributoForm {
    pub tipo_imposto: String,
    pub cst_csosn: Option<String>,
    pub base_calculo: Option<Decimal>,
    pub aliquota: Option<Decimal>,
    pub valor_imposto: Option<Decimal>,
    pub iss_retido: Option<i8>,
    pub classificacao_trib_ibscbs: Option<String>,
    pub aliquota_ibs: Option<Decimal>,
    pub valor_ibs: Option<Decimal>,
    pub aliquota_cbs: Option<Decimal>,
    pub valor_cbs: Option<Decimal>,
}

pub struct ApiTributosController;

#[controller]
impl ApiTributosController {
    pub async fn index(
        mut ctx: doido::controller::Context,
    ) -> doido::controller::Response {
        let access = tenant::tenant_from_ctx(ctx).await.api()?;
        let item_id = parse_id(&ctx, "item_id").api()?;
        tenant_scope::item_for_tenant(ctx.db(), access.empresa.id, item_id).await.api()?;
        let rows = TributoEntity::find()
            .filter(TributoColumn::ItemId.eq(item_id))
            .all(ctx.db())
            .await
            .map_err(|e| ApiError::Internal(e.to_string())).api()?;
        ctx.json(
            rows.into_iter().map(TributoDto::from).collect::<Vec<_>>(),
        )
    }

    pub async fn show(
        mut ctx: doido::controller::Context,
    ) -> doido::controller::Response {
        let access = tenant::tenant_from_ctx(ctx).await.api()?;
        let item_id = parse_id(&ctx, "item_id").api()?;
        let id = parse_id(&ctx, "id").api()?;
        let row =
            tenant_scope::tributo_for_tenant(ctx.db(), access.empresa.id, item_id, id).await.api()?;
        ctx.json(TributoDto::from(row))
    }

    pub async fn create(
        mut ctx: doido::controller::Context,
    ) -> doido::controller::Response {
        let access = tenant::tenant_from_ctx(ctx).await.api()?;
        let item_id = parse_id(&ctx, "item_id").api()?;
        tenant_scope::item_for_tenant(ctx.db(), access.empresa.id, item_id).await.api()?;
        let form: TributoForm = ctx
            .body_json()
            .await
            .map_err(|e| ApiError::Validation(e.to_string())).api()?;
        let record = TributoActive {
            item_id: Set(item_id),
            tipo_imposto: Set(form.tipo_imposto),
            cst_csosn: Set(form.cst_csosn),
            base_calculo: Set(form.base_calculo),
            aliquota: Set(form.aliquota),
            valor_imposto: Set(form.valor_imposto),
            iss_retido: Set(form.iss_retido),
            classificacao_trib_ibscbs: Set(form.classificacao_trib_ibscbs),
            aliquota_ibs: Set(form.aliquota_ibs),
            valor_ibs: Set(form.valor_ibs),
            aliquota_cbs: Set(form.aliquota_cbs),
            valor_cbs: Set(form.valor_cbs),
            ..Default::default()
        };
        let created = record
            .insert(ctx.db())
            .await
            .map_err(|e| ApiError::Internal(e.to_string())).api()?;
        ctx.json(TributoDto::from(created))
    }

    pub async fn update(
        mut ctx: doido::controller::Context,
    ) -> doido::controller::Response {
        let access = tenant::tenant_from_ctx(ctx).await.api()?;
        let item_id = parse_id(&ctx, "item_id").api()?;
        let id = parse_id(&ctx, "id").api()?;
        let existing =
            tenant_scope::tributo_for_tenant(ctx.db(), access.empresa.id, item_id, id).await.api()?;
        let form: TributoForm = ctx
            .body_json()
            .await
            .map_err(|e| ApiError::Validation(e.to_string())).api()?;
        let mut record: TributoActive = existing.into();
        record.tipo_imposto = Set(form.tipo_imposto);
        record.cst_csosn = Set(form.cst_csosn);
        record.base_calculo = Set(form.base_calculo);
        record.aliquota = Set(form.aliquota);
        record.valor_imposto = Set(form.valor_imposto);
        record.iss_retido = Set(form.iss_retido);
        record.classificacao_trib_ibscbs = Set(form.classificacao_trib_ibscbs);
        record.aliquota_ibs = Set(form.aliquota_ibs);
        record.valor_ibs = Set(form.valor_ibs);
        record.aliquota_cbs = Set(form.aliquota_cbs);
        record.valor_cbs = Set(form.valor_cbs);
        let updated = record
            .update(ctx.db())
            .await
            .map_err(|e| ApiError::Internal(e.to_string())).api()?;
        ctx.json(TributoDto::from(updated))
    }

    pub async fn destroy(
        mut ctx: doido::controller::Context,
    ) -> doido::controller::Response {
        let access = tenant::tenant_from_ctx(ctx).await.api()?;
        let item_id = parse_id(&ctx, "item_id").api()?;
        let id = parse_id(&ctx, "id").api()?;
        tenant_scope::tributo_for_tenant(ctx.db(), access.empresa.id, item_id, id).await.api()?;
        TributoEntity::delete_by_id(id)
            .exec(ctx.db())
            .await
            .map_err(|e| ApiError::Internal(e.to_string())).api()?;
        ctx.status(204)
    }
}
