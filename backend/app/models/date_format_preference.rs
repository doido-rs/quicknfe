//! Preferência de formato de data do usuário.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::models::db_enum::normalize_wire;
use crate::services::api_error::ApiError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DateFormatPreference {
    Dmy,
    Mdy,
    Ymd,
}

impl DateFormatPreference {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dmy => "DMY",
            Self::Mdy => "MDY",
            Self::Ymd => "YMD",
        }
    }

    pub fn as_db_code(self) -> i32 {
        match self {
            Self::Dmy => 1,
            Self::Mdy => 2,
            Self::Ymd => 3,
        }
    }

    pub fn from_db_code(code: i32) -> Result<Self, ApiError> {
        match code {
            1 => Ok(Self::Dmy),
            2 => Ok(Self::Mdy),
            3 => Ok(Self::Ymd),
            other => Err(ApiError::Validation(format!(
                "date_format inválido no banco: {other}"
            ))),
        }
    }

    pub fn parse(value: &str) -> Result<Self, ApiError> {
        match normalize_wire(value).as_str() {
            "DMY" => Ok(Self::Dmy),
            "MDY" => Ok(Self::Mdy),
            "YMD" => Ok(Self::Ymd),
            other if other.is_empty() => {
                Err(ApiError::Validation("date_format é obrigatório".into()))
            }
            other => Err(ApiError::Validation(format!(
                "date_format must be DMY, MDY, or YMD (got {other})"
            ))),
        }
    }

    pub fn default() -> Self {
        Self::Dmy
    }
}

impl Serialize for DateFormatPreference {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for DateFormatPreference {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Self::parse(&raw).map_err(serde::de::Error::custom)
    }
}
