use doido::controller::axum::body::Body;
use doido::controller::axum::http::{Request, StatusCode, header};
use doido::controller::axum::Router;
use doido::model::sea_orm::DatabaseConnection;
use tower::ServiceExt;

use super::context::MCP_TOKEN_JTI;
use crate::oauth::store::mcp_session_cookie;

pub fn path_allowed(method: &str, path: &str) -> bool {
    ALLOWLIST
        .iter()
        .any(|(m, p)| *m == method && path_matches(p, path))
}

fn path_matches(template: &str, path: &str) -> bool {
    let t: Vec<&str> = template.trim_matches('/').split('/').collect();
    let p: Vec<&str> = path.trim_matches('/').split('/').collect();
    if t.len() != p.len() {
        return false;
    }
    t.iter()
        .zip(p.iter())
        .all(|(a, b)| a.starts_with('{') && a.ends_with('}') || a == b)
}

const ALLOWLIST: &[(&str, &str)] = &[
    ("GET", "/api/me"),
    ("POST", "/tenant/company"),
    ("POST", "/tenant/prestador"),
    ("GET", "/api/prestadores"),
    ("POST", "/api/prestadores"),
    ("GET", "/api/prestadores/{id}"),
    ("PATCH", "/api/prestadores/{id}"),
    ("DELETE", "/api/prestadores/{id}"),
    ("GET", "/api/clientes"),
    ("POST", "/api/clientes"),
    ("GET", "/api/clientes/{id}"),
    ("PATCH", "/api/clientes/{id}"),
    ("DELETE", "/api/clientes/{id}"),
    ("GET", "/api/intermediarios"),
    ("POST", "/api/intermediarios"),
    ("GET", "/api/intermediarios/{id}"),
    ("PATCH", "/api/intermediarios/{id}"),
    ("DELETE", "/api/intermediarios/{id}"),
    ("GET", "/api/certificados"),
    ("POST", "/api/certificados"),
    ("GET", "/api/certificados/{id}"),
    ("PATCH", "/api/certificados/{id}"),
    ("DELETE", "/api/certificados/{id}"),
    ("GET", "/api/lotes"),
    ("POST", "/api/lotes"),
    ("GET", "/api/lotes/{id}"),
    ("PATCH", "/api/lotes/{id}"),
    ("DELETE", "/api/lotes/{id}"),
    ("GET", "/api/documentos"),
    ("POST", "/api/documentos"),
    ("GET", "/api/documentos/{id}"),
    ("PATCH", "/api/documentos/{id}"),
    ("DELETE", "/api/documentos/{id}"),
    ("POST", "/api/documentos/{id}/transmitir"),
    ("GET", "/api/documentos/{doc_id}/itens"),
    ("POST", "/api/documentos/{doc_id}/itens"),
    ("GET", "/api/documentos/{doc_id}/itens/{id}"),
    ("PATCH", "/api/documentos/{doc_id}/itens/{id}"),
    ("DELETE", "/api/documentos/{doc_id}/itens/{id}"),
    ("GET", "/api/itens/{item_id}/tributos"),
    ("POST", "/api/itens/{item_id}/tributos"),
    ("GET", "/api/itens/{item_id}/tributos/{id}"),
    ("PATCH", "/api/itens/{item_id}/tributos/{id}"),
    ("DELETE", "/api/itens/{item_id}/tributos/{id}"),
    ("GET", "/api/documentos/{doc_id}/eventos"),
    ("POST", "/api/documentos/{doc_id}/eventos"),
    ("GET", "/api/documentos/{doc_id}/eventos/{id}"),
    ("PATCH", "/api/documentos/{doc_id}/eventos/{id}"),
    ("DELETE", "/api/documentos/{doc_id}/eventos/{id}"),
    ("GET", "/api/logs-retorno"),
    ("GET", "/api/logs-retorno/{id}"),
    ("PATCH", "/api/logs-retorno/{id}"),
    ("DELETE", "/api/logs-retorno/{id}"),
];

#[derive(Debug, Clone, serde::Serialize)]
pub struct ApiCallResult {
    pub status: u16,
    pub body: String,
}

pub async fn call_api(
    db: &DatabaseConnection,
    api_router: &Router,
    method: &str,
    path: &str,
    body: Option<&str>,
) -> Result<ApiCallResult, String> {
    if !path.starts_with('/') {
        return Err("path must start with /".into());
    }
    if !path_allowed(method, path) {
        return Err(format!("path not allowed: {method} {path}"));
    }

    let jti = MCP_TOKEN_JTI
        .try_with(|j| j.clone())
        .map_err(|_| "missing MCP auth context".to_string())?;

    let cookie = mcp_session_cookie(db, &jti)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "no QuickNFe session for token".to_string())?;

    let payload = body.unwrap_or("");
    let mut builder = Request::builder()
        .method(method)
        .uri(path)
        .header(header::ACCEPT, "application/json")
        .header(header::COOKIE, cookie);
    if !payload.is_empty() {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
    }
    let req = builder
        .body(Body::from(payload.to_string()))
        .map_err(|e| e.to_string())?;

    let resp = api_router
        .clone()
        .oneshot(req)
        .await
        .map_err(|e| e.to_string())?;
    let status = resp.status();
    let bytes = doido::controller::axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .map_err(|e| e.to_string())?;
    Ok(ApiCallResult {
        status: status.as_u16(),
        body: String::from_utf8_lossy(&bytes).into_owned(),
    })
}

pub fn result_to_text(result: ApiCallResult) -> String {
    serde_json::to_string(&result).unwrap_or_else(|_| {
        format!(
            "{{\"status\":{},\"body\":{}}}",
            result.status,
            serde_json::to_string(&result.body).unwrap_or_default()
        )
    })
}

pub fn is_success(result: &ApiCallResult) -> bool {
    (200..300).contains(&result.status) || result.status == StatusCode::NO_CONTENT.as_u16()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowlist_matches_param_paths() {
        assert!(path_allowed(
            "GET",
            "/api/documentos/9/itens/3"
        ));
        assert!(!path_allowed("GET", "/api/admin/secret"));
    }
}
