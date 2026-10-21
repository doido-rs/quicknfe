//! Modelo do documento fiscal.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::models::db_enum::normalize_wire;
use crate::services::api_error::ApiError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DocumentoModelo {
    Nfse,
}

impl DocumentoModelo {
    pub fn as_str(self) -> &'static str {
        "NFSE"
    }

    pub fn as_db_code(self) -> i32 {
        1
    }

    pub fn from_db_code(code: i32) -> Result<Self, ApiError> {
        match code {
            1 => Ok(Self::Nfse),
            other => Err(ApiError::Validation(format!("modelo inválido: {other}"))),
        }
    }

    pub fn parse(value: &str) -> Result<Self, ApiError> {
        match normalize_wire(value).as_str() {
            "NFSE" => Ok(Self::Nfse),
            other if other.is_empty() => Ok(Self::Nfse),
            other => Err(ApiError::Validation(format!("modelo inválido: {other}"))),
        }
    }
}

impl Serialize for DocumentoModelo {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for DocumentoModelo {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Self::parse(&raw).map_err(serde::de::Error::custom)
    }
}
