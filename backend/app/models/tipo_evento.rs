//! Tipo de evento da nota (cadastro interno; conjunto fechado mínimo).

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::models::db_enum::normalize_wire;
use crate::services::api_error::ApiError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TipoEvento {
    Cancelamento,
    Substituicao,
    Outro,
}

impl TipoEvento {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cancelamento => "CANCELAMENTO",
            Self::Substituicao => "SUBSTITUICAO",
            Self::Outro => "OUTRO",
        }
    }

    pub fn as_db_code(self) -> i32 {
        match self {
            Self::Cancelamento => 1,
            Self::Substituicao => 2,
            Self::Outro => 3,
        }
    }

    pub fn from_db_code(code: i32) -> Result<Self, ApiError> {
        match code {
            1 => Ok(Self::Cancelamento),
            2 => Ok(Self::Substituicao),
            3 => Ok(Self::Outro),
            other => Err(ApiError::Validation(format!("tipo_evento inválido: {other}"))),
        }
    }

    pub fn parse(value: &str) -> Result<Self, ApiError> {
        match normalize_wire(value).as_str() {
            "CANCELAMENTO" => Ok(Self::Cancelamento),
            "SUBSTITUICAO" => Ok(Self::Substituicao),
            "OUTRO" => Ok(Self::Outro),
            other if other.is_empty() => {
                Err(ApiError::Validation("tipo_evento é obrigatório".into()))
            }
            // Legacy free-text: map unknown to OUTRO on parse for migration backfill only;
            // API validation uses strict match below for new writes.
            other => Err(ApiError::Validation(format!(
                "tipo_evento inválido: {other}. Valores: CANCELAMENTO, SUBSTITUICAO, OUTRO"
            ))),
        }
    }

    /// Maps legacy DB strings during migration (non-strict).
    pub fn from_legacy_str(value: &str) -> Self {
        match normalize_wire(value).as_str() {
            "CANCELAMENTO" => Self::Cancelamento,
            "SUBSTITUICAO" => Self::Substituicao,
            _ => Self::Outro,
        }
    }
}

impl Serialize for TipoEvento {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for TipoEvento {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Self::parse(&raw).map_err(serde::de::Error::custom)
    }
}
