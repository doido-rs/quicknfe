//! Tipo de suspensão ISSQN (`tpSusp`).

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::models::db_enum::normalize_wire;
use crate::services::api_error::ApiError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IssqnTpSuspensao {
    Tipo1,
}

impl IssqnTpSuspensao {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Tipo1 => "TIPO_1",
        }
    }

    pub fn as_layout_code(self) -> &'static str {
        "1"
    }

    pub fn as_db_code(self) -> i32 {
        1
    }

    pub fn from_db_code(code: i32) -> Result<Self, ApiError> {
        match code {
            1 => Ok(Self::Tipo1),
            other => Err(ApiError::Validation(format!(
                "issqn_tp_suspensao inválido: {other}"
            ))),
        }
    }

    pub fn parse(value: &str) -> Result<Self, ApiError> {
        let n = normalize_wire(value);
        match n.as_str() {
            "TIPO_1" | "1" => Ok(Self::Tipo1),
            other if other.is_empty() => Ok(Self::Tipo1),
            other => Err(ApiError::Validation(format!(
                "issqn_tp_suspensao inválido: {other}"
            ))),
        }
    }

    pub fn default() -> Self {
        Self::Tipo1
    }
}

impl Serialize for IssqnTpSuspensao {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for IssqnTpSuspensao {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Self::parse(&raw).map_err(serde::de::Error::custom)
    }
}
