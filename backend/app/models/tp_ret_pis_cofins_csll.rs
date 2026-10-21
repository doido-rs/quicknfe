//! Tipo retenção PIS/COFINS/CSLL.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::models::db_enum::normalize_wire;
use crate::services::api_error::ApiError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TpRetPisCofinsCsll {
    NaoRetidos,
    Retidos,
    PisCofinsRetidosCsllNao,
    PisRetido,
    CofinsRetido,
    CofinsCsllRetidosPisNao,
    PisCsllRetidosCofinsNao,
    PisCofinsNaoRetidosCsllRetido,
}

impl TpRetPisCofinsCsll {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NaoRetidos => "NAO_RETIDOS",
            Self::Retidos => "RETIDOS",
            Self::PisCofinsRetidosCsllNao => "PIS_COFINS_RETIDOS_CSLL_NAO",
            Self::PisRetido => "PIS_RETIDO",
            Self::CofinsRetido => "COFINS_RETIDO",
            Self::CofinsCsllRetidosPisNao => "COFINS_CSLL_RETIDOS_PIS_NAO",
            Self::PisCsllRetidosCofinsNao => "PIS_CSLL_RETIDOS_COFINS_NAO",
            Self::PisCofinsNaoRetidosCsllRetido => "PIS_COFINS_NAO_RETIDOS_CSLL_RETIDO",
        }
    }

    pub fn as_layout_code(self) -> &'static str {
        match self {
            Self::NaoRetidos => "0",
            Self::Retidos => "3",
            Self::PisCofinsRetidosCsllNao => "4",
            Self::PisRetido => "5",
            Self::CofinsRetido => "6",
            Self::CofinsCsllRetidosPisNao => "7",
            Self::PisCsllRetidosCofinsNao => "8",
            Self::PisCofinsNaoRetidosCsllRetido => "9",
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
            "NAO_RETIDOS" | "0" => Ok(Self::NaoRetidos),
            "RETIDOS" | "3" => Ok(Self::Retidos),
            "PIS_COFINS_RETIDOS_CSLL_NAO" | "4" => Ok(Self::PisCofinsRetidosCsllNao),
            "PIS_RETIDO" | "5" => Ok(Self::PisRetido),
            "COFINS_RETIDO" | "6" => Ok(Self::CofinsRetido),
            "COFINS_CSLL_RETIDOS_PIS_NAO" | "7" => Ok(Self::CofinsCsllRetidosPisNao),
            "PIS_CSLL_RETIDOS_COFINS_NAO" | "8" => Ok(Self::PisCsllRetidosCofinsNao),
            "PIS_COFINS_NAO_RETIDOS_CSLL_RETIDO" | "9" => Ok(Self::PisCofinsNaoRetidosCsllRetido),
            other if other.is_empty() => Ok(Self::NaoRetidos),
            other => Err(ApiError::Validation(format!(
                "tp_ret_pis_cofins_csll inválido: {other}"
            ))),
        }
    }
}

impl Serialize for TpRetPisCofinsCsll {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for TpRetPisCofinsCsll {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Self::parse(&raw).map_err(serde::de::Error::custom)
    }
}
