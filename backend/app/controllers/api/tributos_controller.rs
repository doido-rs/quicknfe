use doido::controller::controller;
use doido::model::sea_orm::prelude::Decimal;
use doido::model::sea_orm::{ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, Set};
use serde::Deserialize;

use crate::models::tributo_item::{
    ActiveModel as TributoActive, Column as TributoColumn, Entity as TributoEntity,
};
use crate::services::api_error::{db, id_param, json_body, require_tenant, scoped, to_response};
use crate::services::pagination::{page_params_from_ctx, paginate_select};
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
    pub async fn index(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let item_id = crate::api!(id_param(&ctx, "item_id"));
        crate::api!(scoped(tenant_scope::item_for_tenant(ctx.db(), access.prestador.id, item_id).await));
        let params = crate::api!(to_response(page_params_from_ctx(&ctx)));
        let mut select = TributoEntity::find().filter(TributoColumn::ItemId.eq(item_id));
        if let Some(ref q) = params.q {
            select = select.filter(TributoColumn::TipoImposto.contains(q));
        }
        let page = crate::api!(db(
            paginate_select(
                ctx.db(),
                select,
                TributoColumn::Id,
                params,
                |m| m.id,
                TributoDto::from,
            )
            .await,
        ));
        ctx.json(page)
    }

    pub async fn show(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let item_id = crate::api!(id_param(&ctx, "item_id"));
        let id = crate::api!(id_param(&ctx, "id"));
        let row = crate::api!(db(
            tenant_scope::tributo_for_tenant(ctx.db(), access.prestador.id, item_id, id).await,
        ));
        ctx.json(TributoDto::from(row))
    }

    pub async fn create(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let item_id = crate::api!(id_param(&ctx, "item_id"));
        crate::api!(scoped(tenant_scope::item_for_tenant(ctx.db(), access.prestador.id, item_id).await));
        let form: TributoForm = crate::api!(json_body(ctx).await);
        let created = crate::api!(db(
            TributoActive {
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
            }
            .insert(ctx.db())
            .await,
        ));
        ctx.json(TributoDto::from(created))
    }

    pub async fn update(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let item_id = crate::api!(id_param(&ctx, "item_id"));
        let id = crate::api!(id_param(&ctx, "id"));
        let existing = crate::api!(db(
            tenant_scope::tributo_for_tenant(ctx.db(), access.prestador.id, item_id, id).await,
        ));
        let form: TributoForm = crate::api!(json_body(ctx).await);
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
        let updated = crate::api!(db(record.update(ctx.db()).await));
        ctx.json(TributoDto::from(updated))
    }

    pub async fn destroy(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let item_id = crate::api!(id_param(&ctx, "item_id"));
        let id = crate::api!(id_param(&ctx, "id"));
        crate::api!(db(
            tenant_scope::tributo_for_tenant(ctx.db(), access.prestador.id, item_id, id).await,
        ));
        crate::api!(db(TributoEntity::delete_by_id(id).exec(ctx.db()).await));
        ctx.status(204)
    }
}
