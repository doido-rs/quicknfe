//! CST PIS/COFINS.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::models::db_enum::normalize_wire;
use crate::services::api_error::ApiError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CstPisCofins {
    Nenhum,
    AliquotaBasica,
    AliquotaDiferenciada,
    AliquotaUnidade,
    MonofasicaRevendaZero,
    SubstituicaoTributaria,
    AliquotaZero,
    Isenta,
    SemIncidencia,
    ComSuspensao,
    OutrasSaida49,
    Outras99,
}

impl CstPisCofins {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Nenhum => "NENHUM",
            Self::AliquotaBasica => "ALIQUOTA_BASICA",
            Self::AliquotaDiferenciada => "ALIQUOTA_DIFERENCIADA",
            Self::AliquotaUnidade => "ALIQUOTA_UNIDADE",
            Self::MonofasicaRevendaZero => "MONOFASICA_REVENDA_ZERO",
            Self::SubstituicaoTributaria => "SUBSTITUICAO_TRIBUTARIA",
            Self::AliquotaZero => "ALIQUOTA_ZERO",
            Self::Isenta => "ISENTA",
            Self::SemIncidencia => "SEM_INCIDENCIA",
            Self::ComSuspensao => "COM_SUSPENSAO",
            Self::OutrasSaida49 => "OUTRAS_SAIDA_49",
            Self::Outras99 => "OUTRAS_99",
        }
    }

    pub fn as_layout_code(self) -> String {
        match self {
            Self::Nenhum => "00".into(),
            Self::AliquotaBasica => "01".into(),
            Self::AliquotaDiferenciada => "02".into(),
            Self::AliquotaUnidade => "03".into(),
            Self::MonofasicaRevendaZero => "04".into(),
            Self::SubstituicaoTributaria => "05".into(),
            Self::AliquotaZero => "06".into(),
            Self::Isenta => "07".into(),
            Self::SemIncidencia => "08".into(),
            Self::ComSuspensao => "09".into(),
            Self::OutrasSaida49 => "49".into(),
            Self::Outras99 => "99".into(),
        }
    }

    pub fn as_db_code(self) -> i32 {
        match self {
            Self::Nenhum => 0,
            Self::AliquotaBasica => 1,
            Self::AliquotaDiferenciada => 2,
            Self::AliquotaUnidade => 3,
            Self::MonofasicaRevendaZero => 4,
            Self::SubstituicaoTributaria => 5,
            Self::AliquotaZero => 6,
            Self::Isenta => 7,
            Self::SemIncidencia => 8,
            Self::ComSuspensao => 9,
            Self::OutrasSaida49 => 49,
            Self::Outras99 => 99,
        }
    }

    pub fn from_db_code(code: i32) -> Result<Self, ApiError> {
        match code {
            0 => Ok(Self::Nenhum),
            1 => Ok(Self::AliquotaBasica),
            2 => Ok(Self::AliquotaDiferenciada),
            3 => Ok(Self::AliquotaUnidade),
            4 => Ok(Self::MonofasicaRevendaZero),
            5 => Ok(Self::SubstituicaoTributaria),
            6 => Ok(Self::AliquotaZero),
            7 => Ok(Self::Isenta),
            8 => Ok(Self::SemIncidencia),
            9 => Ok(Self::ComSuspensao),
            49 => Ok(Self::OutrasSaida49),
            99 => Ok(Self::Outras99),
            other => Err(ApiError::Validation(format!("cst_pis_cofins inválido: {other}"))),
        }
    }

    pub fn parse(value: &str) -> Result<Self, ApiError> {
        let n = normalize_wire(value);
        let padded = if n.len() == 1 {
            format!("0{n}")
        } else {
            n.clone()
        };
        match n.as_str() {
            "NENHUM" | "00" | "0" => Ok(Self::Nenhum),
            "ALIQUOTA_BASICA" | "01" | "1" => Ok(Self::AliquotaBasica),
            "ALIQUOTA_DIFERENCIADA" | "02" | "2" => Ok(Self::AliquotaDiferenciada),
            "ALIQUOTA_UNIDADE" | "03" | "3" => Ok(Self::AliquotaUnidade),
            "MONOFASICA_REVENDA_ZERO" | "04" | "4" => Ok(Self::MonofasicaRevendaZero),
            "SUBSTITUICAO_TRIBUTARIA" | "05" | "5" => Ok(Self::SubstituicaoTributaria),
            "ALIQUOTA_ZERO" | "06" | "6" => Ok(Self::AliquotaZero),
            "ISENTA" | "07" | "7" => Ok(Self::Isenta),
            "SEM_INCIDENCIA" | "08" | "8" => Ok(Self::SemIncidencia),
            "COM_SUSPENSAO" | "09" | "9" => Ok(Self::ComSuspensao),
            "OUTRAS_SAIDA_49" | "49" => Ok(Self::OutrasSaida49),
            "OUTRAS_99" | "99" => Ok(Self::Outras99),
            _ if padded == "00" => Ok(Self::Nenhum),
            other if other.is_empty() => Ok(Self::Nenhum),
            other => Err(ApiError::Validation(format!("cst_pis_cofins inválido: {other}"))),
        }
    }
}

impl Serialize for CstPisCofins {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for CstPisCofins {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Self::parse(&raw).map_err(serde::de::Error::custom)
    }
}
