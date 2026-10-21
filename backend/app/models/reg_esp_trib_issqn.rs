//! Regime especial de tributação ISSQN (`regEspTrib`).

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::models::db_enum::normalize_wire;
use crate::services::api_error::ApiError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegEspTribIssqn {
    Nenhum,
    MicroempresaMunicipal,
    Estimativa,
    SociedadeProfissionais,
    Cooperativa,
    Mei,
    MeEpp,
}

impl RegEspTribIssqn {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Nenhum => "NENHUM",
            Self::MicroempresaMunicipal => "MICROEMPRESA_MUNICIPAL",
            Self::Estimativa => "ESTIMATIVA",
            Self::SociedadeProfissionais => "SOCIEDADE_PROFISSIONAIS",
            Self::Cooperativa => "COOPERATIVA",
            Self::Mei => "MEI",
            Self::MeEpp => "ME_EPP",
        }
    }

    pub fn as_layout_code(self) -> &'static str {
        match self {
            Self::Nenhum => "0",
            Self::MicroempresaMunicipal => "1",
            Self::Estimativa => "2",
            Self::SociedadeProfissionais => "3",
            Self::Cooperativa => "4",
            Self::Mei => "5",
            Self::MeEpp => "6",
        }
    }

    pub fn as_db_code(self) -> i32 {
        self.as_layout_code().parse().unwrap()
    }

    pub fn from_db_code(code: i32) -> Result<Self, ApiError> {
        Self::parse(&code.to_string())
    }

    pub fn parse(value: &str) -> Result<Self, ApiError> {
        let n = normalize_wire(value);
        match n.as_str() {
            "NENHUM" | "0" => Ok(Self::Nenhum),
            "MICROEMPRESA_MUNICIPAL" | "1" => Ok(Self::MicroempresaMunicipal),
            "ESTIMATIVA" | "2" => Ok(Self::Estimativa),
            "SOCIEDADE_PROFISSIONAIS" | "3" => Ok(Self::SociedadeProfissionais),
            "COOPERATIVA" | "4" => Ok(Self::Cooperativa),
            "MEI" | "5" => Ok(Self::Mei),
            "ME_EPP" | "6" => Ok(Self::MeEpp),
            other if other.is_empty() => {
                Err(ApiError::Validation("reg_esp_trib_issqn é obrigatório".into()))
            }
            other => Err(ApiError::Validation(format!(
                "reg_esp_trib_issqn inválido: {other}"
            ))),
        }
    }
}

impl Serialize for RegEspTribIssqn {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for RegEspTribIssqn {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Self::parse(&raw).map_err(serde::de::Error::custom)
    }
}
