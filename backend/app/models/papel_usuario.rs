//! Papel do usuário na company (tenant membership).

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::models::db_enum::normalize_wire;
use crate::services::api_error::ApiError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PapelUsuario {
    Proprietario,
    Admin,
    Membro,
    Leitura,
}

impl PapelUsuario {
    pub const ALL: [Self; 4] = [
        Self::Proprietario,
        Self::Admin,
        Self::Membro,
        Self::Leitura,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Proprietario => "PROPRIETARIO",
            Self::Admin => "ADMIN",
            Self::Membro => "MEMBRO",
            Self::Leitura => "LEITURA",
        }
    }

    pub fn as_db_code(self) -> i32 {
        match self {
            Self::Proprietario => 1,
            Self::Admin => 2,
            Self::Membro => 3,
            Self::Leitura => 4,
        }
    }

    pub fn from_db_code(code: i32) -> Result<Self, ApiError> {
        match code {
            1 => Ok(Self::Proprietario),
            2 => Ok(Self::Admin),
            3 => Ok(Self::Membro),
            4 => Ok(Self::Leitura),
            other => Err(ApiError::Validation(format!("papel inválido no banco: {other}"))),
        }
    }

    pub fn parse(value: &str) -> Result<Self, ApiError> {
        match normalize_wire(value).as_str() {
            "PROPRIETARIO" => Ok(Self::Proprietario),
            "ADMIN" => Ok(Self::Admin),
            "MEMBRO" => Ok(Self::Membro),
            "LEITURA" => Ok(Self::Leitura),
            other if other.is_empty() => Err(ApiError::Validation("papel é obrigatório".into())),
            other => Err(ApiError::Validation(format!(
                "papel inválido: {other}. Valores: PROPRIETARIO, ADMIN, MEMBRO, LEITURA"
            ))),
        }
    }
}

impl Serialize for PapelUsuario {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for PapelUsuario {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Self::parse(&raw).map_err(serde::de::Error::custom)
    }
}
