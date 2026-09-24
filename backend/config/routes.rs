use crate::controllers::{HelloController, TenantController};
use crate::models::user::Model as User;
use doido::controller::axum;

// Storage serving routes (`/doido/storage/...`) are mounted automatically at
// boot when storage initialises successfully.

pub fn router() -> axum::Router {
    doido::auth::routes! {
        get!("/", HelloController::index);
        auth_routes!(User);
        post!("/tenant/empresa", TenantController::switch);
    }
}
