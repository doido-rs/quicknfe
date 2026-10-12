use doido::controller::controller;
use doido::model::sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter};

use crate::models::cna::{Column as CnaColumn, Entity as CnaEntity};
use crate::services::api_error::{db, id_param, require_tenant, scoped, to_response, ApiError};
use crate::services::dto::CnaDto;
use crate::services::pagination::{page_params_from_ctx, paginate_select};

pub struct ApiCnasController;

#[controller]
impl ApiCnasController {
    pub async fn index(mut ctx: doido::controller::Context) -> doido::controller::Response {
        crate::api!(require_tenant(ctx).await);
        let params = crate::api!(to_response(page_params_from_ctx(&ctx)));
        let mut select = CnaEntity::find();
        if let Some(ref q) = params.q {
            select = select.filter(
                Condition::any()
                    .add(CnaColumn::Codigo.contains(q))
                    .add(CnaColumn::Descricao.contains(q)),
            );
        }
        let page = crate::api!(db(
            paginate_select(
                ctx.db(),
                select,
                CnaColumn::Id,
                params,
                |m| m.id,
                CnaDto::from,
            )
            .await,
        ));
        ctx.json(page)
    }

    pub async fn show(mut ctx: doido::controller::Context) -> doido::controller::Response {
        crate::api!(require_tenant(ctx).await);
        let id = crate::api!(id_param(&ctx, "id"));
        let row = crate::api!(db(CnaEntity::find_by_id(id).one(ctx.db()).await));
        let row = crate::api!(scoped(row.ok_or(ApiError::NotFound)));
        ctx.json(CnaDto::from(row))
    }
}
