use std::sync::Arc;

use doido::controller::axum::Router;
use doido::model::sea_orm::DatabaseConnection;
use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, ContentBlock, ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;

use super::api_client::{call_api, is_success, result_to_text};

#[derive(Clone)]
pub struct McpSharedState {
    pub db: DatabaseConnection,
    pub api_router: Router,
}

#[derive(Clone)]
pub struct QuickNfeMcpServer {
    tool_router: ToolRouter<Self>,
    state: Arc<McpSharedState>,
}

impl QuickNfeMcpServer {
    pub fn new(state: Arc<McpSharedState>) -> Self {
        Self {
            tool_router: Self::tool_router(),
            state,
        }
    }

    async fn api(
        &self,
        method: &str,
        path: String,
        body: Option<serde_json::Value>,
    ) -> CallToolResult {
        let body_str = body.as_ref().map(|v| v.to_string());
        let body_ref = body_str.as_deref();
        match call_api(
            &self.state.db,
            &self.state.api_router,
            method,
            &path,
            body_ref,
        )
        .await
        {
            Ok(result) => {
                let text = result_to_text(result.clone());
                if is_success(&result) {
                    CallToolResult::success(vec![ContentBlock::text(text)])
                } else {
                    CallToolResult::error(vec![ContentBlock::text(text)])
                }
            }
            Err(e) => CallToolResult::error(vec![ContentBlock::text(e)]),
        }
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct IdParam {
    pub id: i64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DocIdParam {
    pub doc_id: i64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DocItemParams {
    pub doc_id: i64,
    pub id: i64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ItemIdParam {
    pub item_id: i64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ItemTributoParams {
    pub item_id: i64,
    pub id: i64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DocEventoParams {
    pub doc_id: i64,
    pub id: i64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct JsonBodyParam {
    #[schemars(description = "JSON request body object")]
    pub body: serde_json::Value,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SwitchEmpresaParam {
    pub empresa_id: i64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TransmitirParam {
    pub id: i64,
    pub certificado_id: Option<i64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct QuicknfeRequestParam {
    pub method: String,
    pub path: String,
    #[serde(default)]
    pub body: Option<serde_json::Value>,
}

#[tool_router(router = tool_router)]
impl QuickNfeMcpServer {
    #[tool(description = "Generic QuickNFe API call (allowlisted paths only)")]
    async fn quicknfe_request(
        &self,
        Parameters(p): Parameters<QuicknfeRequestParam>,
    ) -> CallToolResult {
        self.api(&p.method.to_uppercase(), p.path, p.body).await
    }

    #[tool(description = "GET /api/me — current user and tenant context")]
    async fn quicknfe_me(&self) -> CallToolResult {
        self.api("GET", "/api/me".into(), None).await
    }

    #[tool(description = "POST /tenant/empresa — select active empresa for session")]
    async fn quicknfe_switch_empresa(
        &self,
        Parameters(p): Parameters<SwitchEmpresaParam>,
    ) -> CallToolResult {
        self.api(
            "POST",
            "/tenant/empresa".into(),
            Some(serde_json::json!({ "empresa_id": p.empresa_id })),
        )
        .await
    }

    #[tool(description = "GET /api/empresa")]
    async fn quicknfe_empresa_show(&self) -> CallToolResult {
        self.api("GET", "/api/empresa".into(), None).await
    }

    #[tool(description = "PATCH /api/empresa")]
    async fn quicknfe_empresa_update(
        &self,
        Parameters(p): Parameters<JsonBodyParam>,
    ) -> CallToolResult {
        self.api("PATCH", "/api/empresa".into(), Some(p.body)).await
    }

    #[tool(description = "List clientes")]
    async fn quicknfe_clientes_list(&self) -> CallToolResult {
        self.api("GET", "/api/clientes".into(), None).await
    }

    #[tool(description = "Get cliente by id")]
    async fn quicknfe_clientes_get(&self, Parameters(p): Parameters<IdParam>) -> CallToolResult {
        self.api("GET", format!("/api/clientes/{}", p.id), None).await
    }

    #[tool(description = "Create cliente")]
    async fn quicknfe_clientes_create(
        &self,
        Parameters(p): Parameters<JsonBodyParam>,
    ) -> CallToolResult {
        self.api("POST", "/api/clientes".into(), Some(p.body)).await
    }

    #[tool(description = "Update cliente")]
    async fn quicknfe_clientes_update(
        &self,
        Parameters(p): Parameters<IdBodyParam>,
    ) -> CallToolResult {
        self.api(
            "PATCH",
            format!("/api/clientes/{}", p.id),
            Some(p.body),
        )
        .await
    }

    #[tool(description = "Delete cliente")]
    async fn quicknfe_clientes_delete(&self, Parameters(p): Parameters<IdParam>) -> CallToolResult {
        self.api("DELETE", format!("/api/clientes/{}", p.id), None)
            .await
    }

    #[tool(description = "List intermediarios (notas fiscais intermediarias)")]
    async fn quicknfe_intermediarios_list(&self) -> CallToolResult {
        self.api("GET", "/api/intermediarios".into(), None).await
    }

    #[tool(description = "Get intermediario by id")]
    async fn quicknfe_intermediarios_get(&self, Parameters(p): Parameters<IdParam>) -> CallToolResult {
        self.api("GET", format!("/api/intermediarios/{}", p.id), None)
            .await
    }

    #[tool(description = "Create intermediario")]
    async fn quicknfe_intermediarios_create(
        &self,
        Parameters(p): Parameters<JsonBodyParam>,
    ) -> CallToolResult {
        self.api("POST", "/api/intermediarios".into(), Some(p.body)).await
    }

    #[tool(description = "Update intermediario")]
    async fn quicknfe_intermediarios_update(
        &self,
        Parameters(p): Parameters<IdBodyParam>,
    ) -> CallToolResult {
        self.api(
            "PATCH",
            format!("/api/intermediarios/{}", p.id),
            Some(p.body),
        )
        .await
    }

    #[tool(description = "Delete intermediario")]
    async fn quicknfe_intermediarios_delete(
        &self,
        Parameters(p): Parameters<IdParam>,
    ) -> CallToolResult {
        self.api("DELETE", format!("/api/intermediarios/{}", p.id), None)
            .await
    }

    #[tool(description = "List certificados")]
    async fn quicknfe_certificados_list(&self) -> CallToolResult {
        self.api("GET", "/api/certificados".into(), None).await
    }

    #[tool(description = "Get certificado")]
    async fn quicknfe_certificados_get(&self, Parameters(p): Parameters<IdParam>) -> CallToolResult {
        self.api("GET", format!("/api/certificados/{}", p.id), None)
            .await
    }

    #[tool(description = "Create certificado")]
    async fn quicknfe_certificados_create(
        &self,
        Parameters(p): Parameters<JsonBodyParam>,
    ) -> CallToolResult {
        self.api("POST", "/api/certificados".into(), Some(p.body))
            .await
    }

    #[tool(description = "Update certificado")]
    async fn quicknfe_certificados_update(
        &self,
        Parameters(p): Parameters<IdBodyParam>,
    ) -> CallToolResult {
        self.api(
            "PATCH",
            format!("/api/certificados/{}", p.id),
            Some(p.body),
        )
        .await
    }

    #[tool(description = "Delete certificado")]
    async fn quicknfe_certificados_delete(
        &self,
        Parameters(p): Parameters<IdParam>,
    ) -> CallToolResult {
        self.api("DELETE", format!("/api/certificados/{}", p.id), None)
            .await
    }

    #[tool(description = "List lotes")]
    async fn quicknfe_lotes_list(&self) -> CallToolResult {
        self.api("GET", "/api/lotes".into(), None).await
    }

    #[tool(description = "Get lote")]
    async fn quicknfe_lotes_get(&self, Parameters(p): Parameters<IdParam>) -> CallToolResult {
        self.api("GET", format!("/api/lotes/{}", p.id), None).await
    }

    #[tool(description = "Create lote")]
    async fn quicknfe_lotes_create(
        &self,
        Parameters(p): Parameters<JsonBodyParam>,
    ) -> CallToolResult {
        self.api("POST", "/api/lotes".into(), Some(p.body)).await
    }

    #[tool(description = "Update lote")]
    async fn quicknfe_lotes_update(
        &self,
        Parameters(p): Parameters<IdBodyParam>,
    ) -> CallToolResult {
        self.api("PATCH", format!("/api/lotes/{}", p.id), Some(p.body))
            .await
    }

    #[tool(description = "Delete lote")]
    async fn quicknfe_lotes_delete(&self, Parameters(p): Parameters<IdParam>) -> CallToolResult {
        self.api("DELETE", format!("/api/lotes/{}", p.id), None)
            .await
    }

    #[tool(description = "List documentos fiscais")]
    async fn quicknfe_documentos_list(&self) -> CallToolResult {
        self.api("GET", "/api/documentos".into(), None).await
    }

    #[tool(description = "Get documento")]
    async fn quicknfe_documentos_get(&self, Parameters(p): Parameters<IdParam>) -> CallToolResult {
        self.api("GET", format!("/api/documentos/{}", p.id), None)
            .await
    }

    #[tool(description = "Create documento")]
    async fn quicknfe_documentos_create(
        &self,
        Parameters(p): Parameters<JsonBodyParam>,
    ) -> CallToolResult {
        self.api("POST", "/api/documentos".into(), Some(p.body))
            .await
    }

    #[tool(description = "Update documento")]
    async fn quicknfe_documentos_update(
        &self,
        Parameters(p): Parameters<IdBodyParam>,
    ) -> CallToolResult {
        self.api(
            "PATCH",
            format!("/api/documentos/{}", p.id),
            Some(p.body),
        )
        .await
    }

    #[tool(description = "Delete documento")]
    async fn quicknfe_documentos_delete(
        &self,
        Parameters(p): Parameters<IdParam>,
    ) -> CallToolResult {
        self.api("DELETE", format!("/api/documentos/{}", p.id), None)
            .await
    }

    #[tool(description = "Transmit documento to NFSe")]
    async fn quicknfe_documentos_transmitir(
        &self,
        Parameters(p): Parameters<TransmitirParam>,
    ) -> CallToolResult {
        let body = serde_json::json!({ "certificado_id": p.certificado_id });
        self.api(
            "POST",
            format!("/api/documentos/{}/transmitir", p.id),
            Some(body),
        )
        .await
    }

    #[tool(description = "List itens for documento")]
    async fn quicknfe_itens_list(&self, Parameters(p): Parameters<DocIdParam>) -> CallToolResult {
        self.api(
            "GET",
            format!("/api/documentos/{}/itens", p.doc_id),
            None,
        )
        .await
    }

    #[tool(description = "Get item")]
    async fn quicknfe_itens_get(&self, Parameters(p): Parameters<DocItemParams>) -> CallToolResult {
        self.api(
            "GET",
            format!("/api/documentos/{}/itens/{}", p.doc_id, p.id),
            None,
        )
        .await
    }

    #[tool(description = "Create item")]
    async fn quicknfe_itens_create(
        &self,
        Parameters(p): Parameters<DocIdBodyParam>,
    ) -> CallToolResult {
        self.api(
            "POST",
            format!("/api/documentos/{}/itens", p.doc_id),
            Some(p.body),
        )
        .await
    }

    #[tool(description = "Update item")]
    async fn quicknfe_itens_update(
        &self,
        Parameters(p): Parameters<DocItemBodyParam>,
    ) -> CallToolResult {
        self.api(
            "PATCH",
            format!("/api/documentos/{}/itens/{}", p.doc_id, p.id),
            Some(p.body),
        )
        .await
    }

    #[tool(description = "Delete item")]
    async fn quicknfe_itens_delete(
        &self,
        Parameters(p): Parameters<DocItemParams>,
    ) -> CallToolResult {
        self.api(
            "DELETE",
            format!("/api/documentos/{}/itens/{}", p.doc_id, p.id),
            None,
        )
        .await
    }

    #[tool(description = "List tributos for item")]
    async fn quicknfe_tributos_list(
        &self,
        Parameters(p): Parameters<ItemIdParam>,
    ) -> CallToolResult {
        self.api(
            "GET",
            format!("/api/itens/{}/tributos", p.item_id),
            None,
        )
        .await
    }

    #[tool(description = "Get tributo")]
    async fn quicknfe_tributos_get(
        &self,
        Parameters(p): Parameters<ItemTributoParams>,
    ) -> CallToolResult {
        self.api(
            "GET",
            format!("/api/itens/{}/tributos/{}", p.item_id, p.id),
            None,
        )
        .await
    }

    #[tool(description = "Create tributo")]
    async fn quicknfe_tributos_create(
        &self,
        Parameters(p): Parameters<ItemIdBodyParam>,
    ) -> CallToolResult {
        self.api(
            "POST",
            format!("/api/itens/{}/tributos", p.item_id),
            Some(p.body),
        )
        .await
    }

    #[tool(description = "Update tributo")]
    async fn quicknfe_tributos_update(
        &self,
        Parameters(p): Parameters<ItemTributoBodyParam>,
    ) -> CallToolResult {
        self.api(
            "PATCH",
            format!("/api/itens/{}/tributos/{}", p.item_id, p.id),
            Some(p.body),
        )
        .await
    }

    #[tool(description = "Delete tributo")]
    async fn quicknfe_tributos_delete(
        &self,
        Parameters(p): Parameters<ItemTributoParams>,
    ) -> CallToolResult {
        self.api(
            "DELETE",
            format!("/api/itens/{}/tributos/{}", p.item_id, p.id),
            None,
        )
        .await
    }

    #[tool(description = "List eventos for documento")]
    async fn quicknfe_eventos_list(&self, Parameters(p): Parameters<DocIdParam>) -> CallToolResult {
        self.api(
            "GET",
            format!("/api/documentos/{}/eventos", p.doc_id),
            None,
        )
        .await
    }

    #[tool(description = "Get evento")]
    async fn quicknfe_eventos_get(
        &self,
        Parameters(p): Parameters<DocEventoParams>,
    ) -> CallToolResult {
        self.api(
            "GET",
            format!("/api/documentos/{}/eventos/{}", p.doc_id, p.id),
            None,
        )
        .await
    }

    #[tool(description = "Create evento")]
    async fn quicknfe_eventos_create(
        &self,
        Parameters(p): Parameters<DocIdBodyParam>,
    ) -> CallToolResult {
        self.api(
            "POST",
            format!("/api/documentos/{}/eventos", p.doc_id),
            Some(p.body),
        )
        .await
    }

    #[tool(description = "Update evento")]
    async fn quicknfe_eventos_update(
        &self,
        Parameters(p): Parameters<DocEventoBodyParam>,
    ) -> CallToolResult {
        self.api(
            "PATCH",
            format!("/api/documentos/{}/eventos/{}", p.doc_id, p.id),
            Some(p.body),
        )
        .await
    }

    #[tool(description = "Delete evento")]
    async fn quicknfe_eventos_delete(
        &self,
        Parameters(p): Parameters<DocEventoParams>,
    ) -> CallToolResult {
        self.api(
            "DELETE",
            format!("/api/documentos/{}/eventos/{}", p.doc_id, p.id),
            None,
        )
        .await
    }

    #[tool(description = "List logs retorno")]
    async fn quicknfe_logs_retorno_list(&self) -> CallToolResult {
        self.api("GET", "/api/logs-retorno".into(), None).await
    }

    #[tool(description = "Get log retorno")]
    async fn quicknfe_logs_retorno_get(&self, Parameters(p): Parameters<IdParam>) -> CallToolResult {
        self.api("GET", format!("/api/logs-retorno/{}", p.id), None)
            .await
    }

    #[tool(description = "Update log retorno")]
    async fn quicknfe_logs_retorno_update(
        &self,
        Parameters(p): Parameters<IdBodyParam>,
    ) -> CallToolResult {
        self.api(
            "PATCH",
            format!("/api/logs-retorno/{}", p.id),
            Some(p.body),
        )
        .await
    }

    #[tool(description = "Delete log retorno")]
    async fn quicknfe_logs_retorno_delete(
        &self,
        Parameters(p): Parameters<IdParam>,
    ) -> CallToolResult {
        self.api("DELETE", format!("/api/logs-retorno/{}", p.id), None)
            .await
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct IdBodyParam {
    pub id: i64,
    pub body: serde_json::Value,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DocIdBodyParam {
    pub doc_id: i64,
    pub body: serde_json::Value,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DocItemBodyParam {
    pub doc_id: i64,
    pub id: i64,
    pub body: serde_json::Value,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ItemIdBodyParam {
    pub item_id: i64,
    pub body: serde_json::Value,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ItemTributoBodyParam {
    pub item_id: i64,
    pub id: i64,
    pub body: serde_json::Value,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DocEventoBodyParam {
    pub doc_id: i64,
    pub id: i64,
    pub body: serde_json::Value,
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for QuickNfeMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions(
                "QuickNFe MCP — OAuth-protected tools over the REST API. \
                 Call quicknfe_switch_empresa before tenant-scoped operations.",
            )
    }
}
