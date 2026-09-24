use doido::controller::axum::http::StatusCode;
use doido::controller::axum::response::{IntoResponse, Response};

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
            | ApiError::Tenant(TenantError::NoTenantSelected) => StatusCode::FORBIDDEN,
            ApiError::Tenant(TenantError::Internal(_)) | ApiError::Internal(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            ApiError::NotFound => StatusCode::NOT_FOUND,
            ApiError::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
        };
        (status, self.to_string()).into_response()
    }
}

pub fn parse_id(ctx: &doido::controller::Context, name: &str) -> Result<i64, ApiError> {
    ctx.param(name)
        .and_then(|v| v.parse().ok())
        .ok_or(ApiError::NotFound)
}

pub trait ApiErrExt<T> {
    fn api(self) -> ApiResponse<T>;
}

impl<T, E: Into<ApiError>> ApiErrExt<T> for Result<T, E> {
    fn api(self) -> ApiResponse<T> {
        self.map_err(|e| e.into().into_response())
    }
}

impl<T> ApiErrExt<T> for Result<T, crate::auth::tenant::TenantError> {
    fn api(self) -> ApiResponse<T> {
        self.map_err(|e| e.into_response())
    }
}
