//! Situação da nota fiscal no fluxo assinar/transmitir.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::models::db_enum::normalize_wire;
use crate::services::api_error::ApiError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DocumentoStatus {
    Novo,
    Rascunho,
    Assinando,
    Assinado,
    Transmitindo,
    Autorizada,
    Rejeitada,
    ErroAssinatura,
    ErroTransmissao,
}

impl DocumentoStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Novo => "NOVO",
            Self::Rascunho => "RASCUNHO",
            Self::Assinando => "ASSINANDO",
            Self::Assinado => "ASSINADO",
            Self::Transmitindo => "TRANSMITINDO",
            Self::Autorizada => "AUTORIZADA",
            Self::Rejeitada => "REJEITADA",
            Self::ErroAssinatura => "ERRO_ASSINATURA",
            Self::ErroTransmissao => "ERRO_TRANSMISSAO",
        }
    }

    pub fn as_db_code(self) -> i32 {
        match self {
            Self::Novo => 1,
            Self::Rascunho => 2,
            Self::Assinando => 3,
            Self::Assinado => 4,
            Self::Transmitindo => 5,
            Self::Autorizada => 6,
            Self::Rejeitada => 7,
            Self::ErroAssinatura => 8,
            Self::ErroTransmissao => 9,
        }
    }

    pub fn from_db_code(code: i32) -> Result<Self, ApiError> {
        match code {
            1 => Ok(Self::Novo),
            2 => Ok(Self::Rascunho),
            3 => Ok(Self::Assinando),
            4 => Ok(Self::Assinado),
            5 => Ok(Self::Transmitindo),
            6 => Ok(Self::Autorizada),
            7 => Ok(Self::Rejeitada),
            8 => Ok(Self::ErroAssinatura),
            9 => Ok(Self::ErroTransmissao),
            other => Err(ApiError::Validation(format!(
                "status de documento inválido no banco: {other}"
            ))),
        }
    }

    pub fn parse(value: &str) -> Result<Self, ApiError> {
        match normalize_wire(value).as_str() {
            "NOVO" => Ok(Self::Novo),
            "RASCUNHO" => Ok(Self::Rascunho),
            "ASSINANDO" => Ok(Self::Assinando),
            "ASSINADO" => Ok(Self::Assinado),
            "TRANSMITINDO" => Ok(Self::Transmitindo),
            "AUTORIZADA" | "AUTORIZADO" => Ok(Self::Autorizada),
            "REJEITADA" => Ok(Self::Rejeitada),
            "ERRO_ASSINATURA" => Ok(Self::ErroAssinatura),
            "ERRO_TRANSMISSAO" => Ok(Self::ErroTransmissao),
            other if other.is_empty() => Err(ApiError::Validation("status é obrigatório".into())),
            other => Err(ApiError::Validation(format!("status inválido: {other}"))),
        }
    }

    pub fn is_in_flight(self) -> bool {
        matches!(self, Self::Assinando | Self::Transmitindo)
    }
}

impl Serialize for DocumentoStatus {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for DocumentoStatus {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Self::parse(&raw).map_err(serde::de::Error::custom)
    }
}
