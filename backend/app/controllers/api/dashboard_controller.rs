use doido::controller::controller;

use crate::services::api_error::{db, require_tenant};
use crate::services::dashboard;

pub struct ApiDashboardController;

#[controller]
impl ApiDashboardController {
    pub async fn show(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let stats = crate::api!(db(
            dashboard::load_dashboard_stats(ctx.db(), access.prestador.id).await,
        ));
        ctx.json(stats)
    }
}
