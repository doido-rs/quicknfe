//! Tipo de imposto na linha de tributo do item.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::models::db_enum::normalize_wire;
use crate::services::api_error::ApiError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TipoImposto {
    Iss,
    Pis,
    Cofins,
    Csll,
    Irrf,
    Inss,
    Outro,
}

impl TipoImposto {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Iss => "ISS",
            Self::Pis => "PIS",
            Self::Cofins => "COFINS",
            Self::Csll => "CSLL",
            Self::Irrf => "IRRF",
            Self::Inss => "INSS",
            Self::Outro => "OUTRO",
        }
    }

    pub fn as_db_code(self) -> i32 {
        match self {
            Self::Iss => 1,
            Self::Pis => 2,
            Self::Cofins => 3,
            Self::Csll => 4,
            Self::Irrf => 5,
            Self::Inss => 6,
            Self::Outro => 7,
        }
    }

    pub fn from_db_code(code: i32) -> Result<Self, ApiError> {
        match code {
            1 => Ok(Self::Iss),
            2 => Ok(Self::Pis),
            3 => Ok(Self::Cofins),
            4 => Ok(Self::Csll),
            5 => Ok(Self::Irrf),
            6 => Ok(Self::Inss),
            7 => Ok(Self::Outro),
            other => Err(ApiError::Validation(format!("tipo_imposto inválido: {other}"))),
        }
    }

    pub fn parse(value: &str) -> Result<Self, ApiError> {
        match normalize_wire(value).as_str() {
            "ISS" => Ok(Self::Iss),
            "PIS" => Ok(Self::Pis),
            "COFINS" => Ok(Self::Cofins),
            "CSLL" => Ok(Self::Csll),
            "IRRF" => Ok(Self::Irrf),
            "INSS" => Ok(Self::Inss),
            "OUTRO" => Ok(Self::Outro),
            other if other.is_empty() => {
                Err(ApiError::Validation("tipo_imposto é obrigatório".into()))
            }
            other => Err(ApiError::Validation(format!("tipo_imposto inválido: {other}"))),
        }
    }

    pub fn from_legacy_str(value: &str) -> Self {
        TipoImposto::parse(value).unwrap_or(Self::Outro)
    }
}

impl Serialize for TipoImposto {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for TipoImposto {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Self::parse(&raw).map_err(serde::de::Error::custom)
    }
}
