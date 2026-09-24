use crate::controllers::{
    ApiCertificadosController, ApiClientesController, ApiDocumentosController, ApiEmpresaController,
    ApiEventosController, ApiItensController, ApiLogsController, ApiLotesController,
    ApiSessionController, ApiTributosController, HelloController, TenantController,
};
use crate::models::user::Model as User;
use doido::controller::axum;

// Storage serving routes (`/doido/storage/...`) are mounted automatically at
// boot when storage initialises successfully.

pub fn router() -> axum::Router {
    doido::auth::routes! {
        get!("/", HelloController::index);
        auth_routes!(User);
        post!("/tenant/empresa", TenantController::switch);

        get!("/api/me", ApiSessionController::me);
        get!("/api/empresa", ApiEmpresaController::show);
        patch!("/api/empresa", ApiEmpresaController::update);

        get!("/api/clientes", ApiClientesController::index);
        post!("/api/clientes", ApiClientesController::create);
        get!("/api/clientes/{id}", ApiClientesController::show);
        patch!("/api/clientes/{id}", ApiClientesController::update);
        delete!("/api/clientes/{id}", ApiClientesController::destroy);

        get!("/api/certificados", ApiCertificadosController::index);
        post!("/api/certificados", ApiCertificadosController::create);
        get!("/api/certificados/{id}", ApiCertificadosController::show);
        patch!("/api/certificados/{id}", ApiCertificadosController::update);
        delete!("/api/certificados/{id}", ApiCertificadosController::destroy);

        get!("/api/lotes", ApiLotesController::index);
        post!("/api/lotes", ApiLotesController::create);
        get!("/api/lotes/{id}", ApiLotesController::show);
        patch!("/api/lotes/{id}", ApiLotesController::update);
        delete!("/api/lotes/{id}", ApiLotesController::destroy);

        get!("/api/documentos", ApiDocumentosController::index);
        post!("/api/documentos", ApiDocumentosController::create);
        get!("/api/documentos/{id}", ApiDocumentosController::show);
        patch!("/api/documentos/{id}", ApiDocumentosController::update);
        delete!("/api/documentos/{id}", ApiDocumentosController::destroy);

        get!("/api/documentos/{doc_id}/itens", ApiItensController::index);
        post!("/api/documentos/{doc_id}/itens", ApiItensController::create);
        get!("/api/documentos/{doc_id}/itens/{id}", ApiItensController::show);
        patch!("/api/documentos/{doc_id}/itens/{id}", ApiItensController::update);
        delete!("/api/documentos/{doc_id}/itens/{id}", ApiItensController::destroy);

        get!("/api/itens/{item_id}/tributos", ApiTributosController::index);
        post!("/api/itens/{item_id}/tributos", ApiTributosController::create);
        get!("/api/itens/{item_id}/tributos/{id}", ApiTributosController::show);
        patch!("/api/itens/{item_id}/tributos/{id}", ApiTributosController::update);
        delete!("/api/itens/{item_id}/tributos/{id}", ApiTributosController::destroy);

        get!("/api/documentos/{doc_id}/eventos", ApiEventosController::index);
        post!("/api/documentos/{doc_id}/eventos", ApiEventosController::create);
        get!("/api/documentos/{doc_id}/eventos/{id}", ApiEventosController::show);
        patch!("/api/documentos/{doc_id}/eventos/{id}", ApiEventosController::update);
        delete!("/api/documentos/{doc_id}/eventos/{id}", ApiEventosController::destroy);

        get!("/api/logs-retorno", ApiLogsController::index);
        get!("/api/logs-retorno/{id}", ApiLogsController::show);
        patch!("/api/logs-retorno/{id}", ApiLogsController::update);
        delete!("/api/logs-retorno/{id}", ApiLogsController::destroy);
    }
}
