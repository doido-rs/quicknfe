//! Tipo de certificado digital (A1, PFX, …).

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::models::db_enum::normalize_wire;
use crate::services::api_error::ApiError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CertificadoTipo {
    A1,
    A3,
    Pfx,
    Pem,
}

impl CertificadoTipo {
    pub const ALL: [Self; 4] = [Self::A1, Self::A3, Self::Pfx, Self::Pem];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::A1 => "A1",
            Self::A3 => "A3",
            Self::Pfx => "PFX",
            Self::Pem => "PEM",
        }
    }

    pub fn as_db_code(self) -> i32 {
        match self {
            Self::A1 => 1,
            Self::A3 => 2,
            Self::Pfx => 3,
            Self::Pem => 4,
        }
    }

    pub fn from_db_code(code: i32) -> Result<Self, ApiError> {
        match code {
            1 => Ok(Self::A1),
            2 => Ok(Self::A3),
            3 => Ok(Self::Pfx),
            4 => Ok(Self::Pem),
            other => Err(ApiError::Validation(format!(
                "tipo de certificado inválido no banco: {other}"
            ))),
        }
    }

    pub fn parse(value: &str) -> Result<Self, ApiError> {
        match normalize_wire(value).as_str() {
            "A1" => Ok(Self::A1),
            "A3" => Ok(Self::A3),
            "PFX" => Ok(Self::Pfx),
            "PEM" => Ok(Self::Pem),
            other if other.is_empty() => {
                Err(ApiError::Validation("tipo de certificado é obrigatório".into()))
            }
            other => Err(ApiError::Validation(format!(
                "tipo must be one of: A1, A3, PFX, PEM (got {other})"
            ))),
        }
    }
}

impl Serialize for CertificadoTipo {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for CertificadoTipo {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Self::parse(&raw).map_err(serde::de::Error::custom)
    }
}

pub fn validar_certificado_tipo(tipo: CertificadoTipo) -> Result<(), ApiError> {
    let _ = tipo;
    Ok(())
}
