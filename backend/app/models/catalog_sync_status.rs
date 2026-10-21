//! Status de execução de sync do catálogo NFS-e.

use crate::services::api_error::ApiError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CatalogSyncStatus {
    Running,
    Ok,
    Error,
}

impl CatalogSyncStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Running => "RUNNING",
            Self::Ok => "OK",
            Self::Error => "ERROR",
        }
    }

    pub fn as_db_code(self) -> i32 {
        match self {
            Self::Running => 1,
            Self::Ok => 2,
            Self::Error => 3,
        }
    }

    pub fn from_db_code(code: i32) -> Result<Self, ApiError> {
        match code {
            1 => Ok(Self::Running),
            2 => Ok(Self::Ok),
            3 => Ok(Self::Error),
            other => Err(ApiError::Validation(format!(
                "catalog status inválido: {other}"
            ))),
        }
    }

    pub fn parse(value: &str) -> Result<Self, ApiError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "running" => Ok(Self::Running),
            "ok" | "enqueued" => Ok(Self::Ok),
            "error" | "erro" => Ok(Self::Error),
            other => Err(ApiError::Validation(format!("catalog status inválido: {other}"))),
        }
    }
}
