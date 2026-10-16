use doido::controller::controller;
use doido::model::sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter};
use chrono::NaiveDate;
use serde::Deserialize;

use crate::models::municipio::{Column as MunicipioColumn, Entity as MunicipioEntity};
use crate::services::api_error::{db, id_param, require_tenant, scoped, to_response, ApiError};
use crate::services::dto::MunicipioDto;
use crate::services::pagination::{page_params_from_ctx, paginate_select};

pub struct ApiMunicipiosController;

#[derive(Debug, Deserialize, Default)]
struct MunicipioCnasQuery {
    competencia: Option<String>,
}

#[controller]
impl ApiMunicipiosController {
    pub async fn index(mut ctx: doido::controller::Context) -> doido::controller::Response {
        crate::api!(require_tenant(ctx).await);
        let params = crate::api!(to_response(page_params_from_ctx(&ctx)));
        let mut select = MunicipioEntity::find();
        if let Some(ref q) = params.q {
            select = select.filter(
                Condition::any()
                    .add(MunicipioColumn::Nome.contains(q))
                    .add(MunicipioColumn::CodigoIbge.contains(q))
                    .add(MunicipioColumn::Uf.eq(q.to_uppercase())),
            );
        }
        let page = crate::api!(db(
            paginate_select(
                ctx.db(),
                select,
                MunicipioColumn::Id,
                params,
                |m| m.id,
                MunicipioDto::from,
            )
            .await,
        ));
        ctx.json(page)
    }

    pub async fn show(mut ctx: doido::controller::Context) -> doido::controller::Response {
        crate::api!(require_tenant(ctx).await);
        let id = crate::api!(id_param(&ctx, "id"));
        let row = crate::api!(db(MunicipioEntity::find_by_id(id).one(ctx.db()).await));
        let row = crate::api!(scoped(row.ok_or(ApiError::NotFound)));
        ctx.json(MunicipioDto::from(row))
    }

    /// CNAs administrados pelo município na competência (parametrização ADN sincronizada).
    pub async fn cnas(mut ctx: doido::controller::Context) -> doido::controller::Response {
        crate::api!(require_tenant(ctx).await);
        let id = crate::api!(id_param(&ctx, "id"));
        let row = crate::api!(db(MunicipioEntity::find_by_id(id).one(ctx.db()).await));
        let _row = crate::api!(scoped(row.ok_or(ApiError::NotFound)));

        let query: MunicipioCnasQuery = ctx
            .query_params()
            .deserialize()
            .unwrap_or_default();
        let competencia = query
            .competencia
            .as_deref()
            .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

        let list = crate::api!(to_response(
            crate::services::nfse::list_cnas_administrados(
                ctx.db(),
                id,
                competencia,
            )
            .await,
        ));
        ctx.json(list)
    }
}
