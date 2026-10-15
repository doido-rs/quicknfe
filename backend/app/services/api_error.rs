use doido::controller::axum::body::Body;
use doido::controller::axum::http::{header, StatusCode};
use doido::controller::axum::response::{IntoResponse, Response};
use serde::Serialize;

use crate::auth::tenant::TenantError;

#[derive(Debug)]
pub enum ApiError {
    Tenant(TenantError),
    NotFound,
    Validation(String),
    Internal(String),
}

impl From<TenantError> for ApiError {
    fn from(value: TenantError) -> Self {
        ApiError::Tenant(value)
    }
}

pub async fn require_tenant(
    ctx: &mut doido::controller::Context,
) -> ApiResponse<crate::auth::tenant::TenantAccess> {
    to_response(crate::auth::tenant::tenant_from_ctx(ctx).await)
}

pub async fn require_user(
    ctx: &mut doido::controller::Context,
) -> ApiResponse<crate::models::user::Model> {
    to_response(crate::auth::tenant::user_from_ctx(ctx).await)
}

pub async fn json_body<T: serde::de::DeserializeOwned>(
    ctx: &mut doido::controller::Context,
) -> ApiResponse<T> {
    to_response(
        ctx.body_json()
            .await
            .map_err(|e| ApiError::Validation(e.to_string())),
    )
}

pub fn db<T, E: std::fmt::Display>(result: Result<T, E>) -> ApiResponse<T> {
    to_response(result.map_err(|e| ApiError::Internal(e.to_string())))
}

/// Erros de escopo tenant (`ApiError`) já mapeiam status HTTP corretos.
pub fn scoped<T>(result: Result<T, ApiError>) -> ApiResponse<T> {
    to_response(result)
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::Tenant(e) => write!(f, "{e}"),
            ApiError::NotFound => write!(f, "not found"),
            ApiError::Validation(msg) => write!(f, "{msg}"),
            ApiError::Internal(msg) => write!(f, "{msg}"),
        }
    }
}

pub type ApiResponse<T> = Result<T, doido::controller::Response>;

pub fn to_response<T, E: Into<ApiError>>(result: Result<T, E>) -> ApiResponse<T> {
    result.map_err(|e| e.into().into_response())
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match &self {
            ApiError::Tenant(TenantError::Unauthorized) => StatusCode::UNAUTHORIZED,
            ApiError::Tenant(TenantError::Forbidden)
            | ApiError::Tenant(TenantError::NoTenantSelected)
            | ApiError::Tenant(TenantError::NoPrestadorSelected) => StatusCode::FORBIDDEN,
            ApiError::Tenant(TenantError::Internal(_)) | ApiError::Internal(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            ApiError::NotFound => StatusCode::NOT_FOUND,
            ApiError::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
        };
        (status, self.to_string()).into_response()
    }
}

pub fn id_param(ctx: &doido::controller::Context, name: &str) -> ApiResponse<i64> {
    to_response(parse_id(ctx, name))
}

pub fn parse_id(ctx: &doido::controller::Context, name: &str) -> Result<i64, ApiError> {
    ctx.param(name)
        .and_then(|v| v.parse().ok())
        .ok_or(ApiError::NotFound)
}

/// Early-return helper for handlers that return [`Response`](doido::controller::Response).
#[macro_export]
macro_rules! api {
    ($expr:expr) => {
        match $expr {
            Ok(__api_ok) => __api_ok,
            Err(__api_err) => return __api_err,
        }
    };
}

pub fn json_with_status<T: Serialize>(status: u16, data: T) -> Response {
    let body = serde_json::to_vec(&data).unwrap_or_default();
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body))
        .expect("valid json response")
}

pub trait ApiErrExt<T> {
    fn api(self) -> ApiResponse<T>;
}

impl<T, E: Into<ApiError>> ApiErrExt<T> for Result<T, E> {
    fn api(self) -> ApiResponse<T> {
        self.map_err(|e| e.into().into_response())
    }
}
