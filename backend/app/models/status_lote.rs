//! Status do lote de transmissão.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::models::db_enum::normalize_wire;
use crate::services::api_error::ApiError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatusLote {
    Aberto,
    Enviado,
    Processado,
    Erro,
}

impl StatusLote {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Aberto => "ABERTO",
            Self::Enviado => "ENVIADO",
            Self::Processado => "PROCESSADO",
            Self::Erro => "ERRO",
        }
    }

    pub fn as_db_code(self) -> i32 {
        match self {
            Self::Aberto => 1,
            Self::Enviado => 2,
            Self::Processado => 3,
            Self::Erro => 4,
        }
    }

    pub fn from_db_code(code: i32) -> Result<Self, ApiError> {
        match code {
            1 => Ok(Self::Aberto),
            2 => Ok(Self::Enviado),
            3 => Ok(Self::Processado),
            4 => Ok(Self::Erro),
            other => Err(ApiError::Validation(format!("status_lote inválido: {other}"))),
        }
    }

    pub fn parse(value: &str) -> Result<Self, ApiError> {
        match normalize_wire(value).as_str() {
            "ABERTO" => Ok(Self::Aberto),
            "ENVIADO" => Ok(Self::Enviado),
            "PROCESSADO" => Ok(Self::Processado),
            "ERRO" => Ok(Self::Erro),
            other if other.is_empty() => {
                Err(ApiError::Validation("status_lote é obrigatório".into()))
            }
            other => Err(ApiError::Validation(format!("status_lote inválido: {other}"))),
        }
    }
}

impl Serialize for StatusLote {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for StatusLote {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Self::parse(&raw).map_err(serde::de::Error::custom)
    }
}
