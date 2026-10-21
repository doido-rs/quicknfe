//! Tipo de execução de sync do catálogo NFS-e.

use crate::services::api_error::ApiError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CatalogSyncKind {
    Nacional,
    Municipios,
    AnexoA,
}

impl CatalogSyncKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Nacional => "NACIONAL",
            Self::Municipios => "MUNICIPIOS",
            Self::AnexoA => "ANEXO_A",
        }
    }

    pub fn as_db_code(self) -> i32 {
        match self {
            Self::Nacional => 1,
            Self::Municipios => 2,
            Self::AnexoA => 3,
        }
    }

    pub fn from_db_code(code: i32) -> Result<Self, ApiError> {
        match code {
            1 => Ok(Self::Nacional),
            2 => Ok(Self::Municipios),
            3 => Ok(Self::AnexoA),
            other => Err(ApiError::Validation(format!("catalog kind inválido: {other}"))),
        }
    }

    pub fn parse(value: &str) -> Result<Self, ApiError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "nacional" => Ok(Self::Nacional),
            "municipios" => Ok(Self::Municipios),
            "anexo_a" | "anexoa" => Ok(Self::AnexoA),
            other => Err(ApiError::Validation(format!("catalog kind inválido: {other}"))),
        }
    }
}
