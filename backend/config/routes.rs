use crate::controllers::{
    ApiCertificadosController, ApiCncController, ApiClientesController, ApiDashboardController,
    ApiDocumentosController,
    ApiEventosController, ApiIntermediariosController, ApiItensController, ApiLogsController,
    ApiPrestadoresController,
    ApiCnasController, ApiMunicipiosController,
    ApiLotesController,
    ApiSessionController, ApiTributosController, HelloController, TenantController,
};
use crate::models::user::Model as User;
use doido::controller::axum;

// Storage serving routes (`/doido/storage/...`) are mounted automatically at
// boot when storage initialises successfully.

pub fn router() -> axum::Router {
    let api = doido::auth::routes! {
        get!("/", HelloController::index);
        auth_routes!(User);
        post!("/tenant/company", TenantController::switch_company);
        post!("/tenant/prestador", TenantController::switch_prestador);

        get!("/api/me", ApiSessionController::me);
        patch!("/api/me", ApiSessionController::update_me);
        get!("/api/dashboard", ApiDashboardController::show);
        get!("/api/prestadores", ApiPrestadoresController::index);
        post!("/api/prestadores", ApiPrestadoresController::create);
        get!("/api/prestadores/{id}", ApiPrestadoresController::show);
        patch!("/api/prestadores/{id}", ApiPrestadoresController::update);
        delete!("/api/prestadores/{id}", ApiPrestadoresController::destroy);

        get!("/api/municipios", ApiMunicipiosController::index);
        get!("/api/municipios/{id}", ApiMunicipiosController::show);

        get!("/api/cnas", ApiCnasController::index);
        get!("/api/cnas/{id}", ApiCnasController::show);

        get!("/api/cnc/cad", ApiCncController::consulta_cad);

        get!("/api/clientes", ApiClientesController::index);
        post!("/api/clientes", ApiClientesController::create);
        get!("/api/clientes/{id}", ApiClientesController::show);
        patch!("/api/clientes/{id}", ApiClientesController::update);
        delete!("/api/clientes/{id}", ApiClientesController::destroy);

        get!("/api/intermediarios", ApiIntermediariosController::index);
        post!("/api/intermediarios", ApiIntermediariosController::create);
        get!("/api/intermediarios/{id}", ApiIntermediariosController::show);
        patch!("/api/intermediarios/{id}", ApiIntermediariosController::update);
        delete!("/api/intermediarios/{id}", ApiIntermediariosController::destroy);

        get!(
            "/api/prestadores/{prestador_id}/certificados",
            ApiCertificadosController::index
        );
        post!(
            "/api/prestadores/{prestador_id}/certificados",
            ApiCertificadosController::create
        );
        get!(
            "/api/prestadores/{prestador_id}/certificados/{id}",
            ApiCertificadosController::show
        );
        patch!(
            "/api/prestadores/{prestador_id}/certificados/{id}",
            ApiCertificadosController::update
        );
        delete!(
            "/api/prestadores/{prestador_id}/certificados/{id}",
            ApiCertificadosController::destroy
        );

        get!("/api/lotes", ApiLotesController::index);
        post!("/api/lotes", ApiLotesController::create);
        get!("/api/lotes/{id}", ApiLotesController::show);
        patch!("/api/lotes/{id}", ApiLotesController::update);
        delete!("/api/lotes/{id}", ApiLotesController::destroy);
        post!("/api/lotes/{id}/transmitir", ApiLotesController::transmitir);

        get!("/api/documentos", ApiDocumentosController::index);
        post!("/api/documentos", ApiDocumentosController::create);
        get!("/api/documentos/{id}", ApiDocumentosController::show);
        get!("/api/documentos/{id}/xml", ApiDocumentosController::download_xml);
        patch!("/api/documentos/{id}", ApiDocumentosController::update);
        delete!("/api/documentos/{id}", ApiDocumentosController::destroy);
        post!(
            "/api/documentos/{id}/transmitir",
            ApiDocumentosController::transmitir
        );
        post!("/api/documentos/{id}/assinar", ApiDocumentosController::assinar);

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

        get!(
            "/api/documentos/{doc_id}/logs-retorno",
            ApiLogsController::index_for_documento
        );
        get!(
            "/api/documentos/{doc_id}/logs-retorno/{id}",
            ApiLogsController::show_for_documento
        );

        get!("/api/logs-retorno", ApiLogsController::index);
        get!("/api/logs-retorno/{id}", ApiLogsController::show);
        patch!("/api/logs-retorno/{id}", ApiLogsController::update);
        delete!("/api/logs-retorno/{id}", ApiLogsController::destroy);
    };

    let api = api.merge(crate::api_docs::router());

    if crate::mcp::enabled() {
        crate::mcp::McpConfig::init();
        let api_for_mcp = api.clone();
        api.merge(crate::oauth::router(api_for_mcp.clone()))
            .merge(crate::mcp::router(api_for_mcp))
    } else {
        api
    }
}
