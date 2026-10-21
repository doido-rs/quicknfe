//! Tributação ISSQN (`tribISSQN`).

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::models::db_enum::normalize_wire;
use crate::services::api_error::ApiError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TribIssqn {
    OperacaoTributavel,
    Imunidade,
    ExportacaoServico,
    NaoIncidencia,
}

impl TribIssqn {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OperacaoTributavel => "OPERACAO_TRIBUTAVEL",
            Self::Imunidade => "IMUNIDADE",
            Self::ExportacaoServico => "EXPORTACAO_SERVICO",
            Self::NaoIncidencia => "NAO_INCIDENCIA",
        }
    }

    pub fn as_layout_code(self) -> &'static str {
        match self {
            Self::OperacaoTributavel => "1",
            Self::Imunidade => "2",
            Self::ExportacaoServico => "3",
            Self::NaoIncidencia => "4",
        }
    }

    pub fn as_db_code(self) -> i32 {
        match self {
            Self::OperacaoTributavel => 1,
            Self::Imunidade => 2,
            Self::ExportacaoServico => 3,
            Self::NaoIncidencia => 4,
        }
    }

    pub fn from_db_code(code: i32) -> Result<Self, ApiError> {
        match code {
            1 => Ok(Self::OperacaoTributavel),
            2 => Ok(Self::Imunidade),
            3 => Ok(Self::ExportacaoServico),
            4 => Ok(Self::NaoIncidencia),
            other => Err(ApiError::Validation(format!("trib_issqn inválido: {other}"))),
        }
    }

    pub fn parse(value: &str) -> Result<Self, ApiError> {
        let n = normalize_wire(value);
        match n.as_str() {
            "OPERACAO_TRIBUTAVEL" | "1" => Ok(Self::OperacaoTributavel),
            "IMUNIDADE" | "2" => Ok(Self::Imunidade),
            "EXPORTACAO_SERVICO" | "3" => Ok(Self::ExportacaoServico),
            "NAO_INCIDENCIA" | "4" => Ok(Self::NaoIncidencia),
            other if other.is_empty() => Err(ApiError::Validation("trib_issqn é obrigatório".into())),
            other => Err(ApiError::Validation(format!("trib_issqn inválido: {other}"))),
        }
    }
}

impl Serialize for TribIssqn {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for TribIssqn {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Self::parse(&raw).map_err(serde::de::Error::custom)
    }
}
