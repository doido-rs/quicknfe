//! Tomador/intermediário: Brasil, exterior ou não informado.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::models::db_enum::normalize_wire;
use crate::services::api_error::ApiError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LocalidadeParte {
    Brasil,
    Exterior,
    NaoInformado,
}

impl LocalidadeParte {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Brasil => "BRASIL",
            Self::Exterior => "EXTERIOR",
            Self::NaoInformado => "NAO_INFORMADO",
        }
    }

    pub fn as_db_code(self) -> i32 {
        match self {
            Self::Brasil => 1,
            Self::Exterior => 2,
            Self::NaoInformado => 3,
        }
    }

    pub fn from_db_code(code: i32) -> Result<Self, ApiError> {
        match code {
            1 => Ok(Self::Brasil),
            2 => Ok(Self::Exterior),
            3 => Ok(Self::NaoInformado),
            other => Err(ApiError::Validation(format!(
                "localidade inválida no banco: {other}"
            ))),
        }
    }

    pub fn parse(value: &str) -> Result<Self, ApiError> {
        match normalize_wire(value).as_str() {
            "BRASIL" => Ok(Self::Brasil),
            "EXTERIOR" => Ok(Self::Exterior),
            "NAO_INFORMADO" => Ok(Self::NaoInformado),
            other if other.is_empty() => {
                Err(ApiError::Validation("localidade é obrigatória".into()))
            }
            other => Err(ApiError::Validation(format!("localidade inválida: {other}"))),
        }
    }
}

impl Serialize for LocalidadeParte {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for LocalidadeParte {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Self::parse(&raw).map_err(serde::de::Error::custom)
    }
}
