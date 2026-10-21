//! Regimes tributários do prestador (CRT NF-e / classificação fiscal).

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::services::api_error::ApiError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegimeTributario {
    MicroempreendedorIndividual,
    SimplesNacional,
    SimplesNacionalExcessoSublimite,
    LucroPresumido,
    LucroReal,
    LucroArbitrado,
    RegimeNormal,
    Imunidade,
    Isencao,
}

impl RegimeTributario {
    pub const ALL: [RegimeTributario; 9] = [
        Self::MicroempreendedorIndividual,
        Self::SimplesNacional,
        Self::SimplesNacionalExcessoSublimite,
        Self::LucroPresumido,
        Self::LucroReal,
        Self::LucroArbitrado,
        Self::RegimeNormal,
        Self::Imunidade,
        Self::Isencao,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::MicroempreendedorIndividual => "MICROEMPREENDEDOR_INDIVIDUAL",
            Self::SimplesNacional => "SIMPLES_NACIONAL",
            Self::SimplesNacionalExcessoSublimite => "SIMPLES_NACIONAL_EXCESSO_SUBLIMITE",
            Self::LucroPresumido => "LUCRO_PRESUMIDO",
            Self::LucroReal => "LUCRO_REAL",
            Self::LucroArbitrado => "LUCRO_ARBITRADO",
            Self::RegimeNormal => "REGIME_NORMAL",
            Self::Imunidade => "IMUNIDADE",
            Self::Isencao => "ISENCAO",
        }
    }

    pub fn label_pt(self) -> &'static str {
        match self {
            Self::MicroempreendedorIndividual => "Microempreendedor Individual (MEI)",
            Self::SimplesNacional => "Simples Nacional",
            Self::SimplesNacionalExcessoSublimite => {
                "Simples Nacional — excesso de sublimite de receita bruta"
            }
            Self::LucroPresumido => "Lucro Presumido",
            Self::LucroReal => "Lucro Real",
            Self::LucroArbitrado => "Lucro Arbitrado",
            Self::RegimeNormal => "Regime Normal",
            Self::Imunidade => "Imunidade",
            Self::Isencao => "Isenção",
        }
    }

    /// Código CRT da NF-e (1, 2 ou 3), quando aplicável.
    pub fn crt_nfe(self) -> Option<i32> {
        match self {
            Self::MicroempreendedorIndividual | Self::SimplesNacional => Some(1),
            Self::SimplesNacionalExcessoSublimite => Some(2),
            Self::LucroPresumido
            | Self::LucroReal
            | Self::LucroArbitrado
            | Self::RegimeNormal => Some(3),
            Self::Imunidade | Self::Isencao => None,
        }
    }

    pub fn parse(value: &str) -> Result<Self, ApiError> {
        match value.trim().to_ascii_uppercase().as_str() {
            "MICROEMPREENDEDOR_INDIVIDUAL" | "MEI" => Ok(Self::MicroempreendedorIndividual),
            "SIMPLES_NACIONAL" => Ok(Self::SimplesNacional),
            "SIMPLES_NACIONAL_EXCESSO_SUBLIMITE" => Ok(Self::SimplesNacionalExcessoSublimite),
            "LUCRO_PRESUMIDO" => Ok(Self::LucroPresumido),
            "LUCRO_REAL" => Ok(Self::LucroReal),
            "LUCRO_ARBITRADO" => Ok(Self::LucroArbitrado),
            "REGIME_NORMAL" => Ok(Self::RegimeNormal),
            "IMUNIDADE" => Ok(Self::Imunidade),
            "ISENCAO" | "ISENTO" => Ok(Self::Isencao),
            other if other.is_empty() => Err(ApiError::Validation(
                "regime_tributario é obrigatório".into(),
            )),
            other => Err(ApiError::Validation(format!(
                "regime_tributario inválido: {other}. Valores: {}",
                Self::ALL
                    .iter()
                    .map(|r| r.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))),
        }
    }

    pub fn from_legacy_crt(codigo: i32) -> Self {
        match codigo {
            1 => Self::SimplesNacional,
            2 => Self::SimplesNacionalExcessoSublimite,
            3 => Self::RegimeNormal,
            _ => Self::RegimeNormal,
        }
    }

    pub fn as_db_code(self) -> i32 {
        match self {
            Self::MicroempreendedorIndividual => 1,
            Self::SimplesNacional => 2,
            Self::SimplesNacionalExcessoSublimite => 3,
            Self::LucroPresumido => 4,
            Self::LucroReal => 5,
            Self::LucroArbitrado => 6,
            Self::RegimeNormal => 7,
            Self::Imunidade => 8,
            Self::Isencao => 9,
        }
    }

    pub fn from_db_code(code: i32) -> Result<Self, ApiError> {
        match code {
            1 => Ok(Self::MicroempreendedorIndividual),
            2 => Ok(Self::SimplesNacional),
            3 => Ok(Self::SimplesNacionalExcessoSublimite),
            4 => Ok(Self::LucroPresumido),
            5 => Ok(Self::LucroReal),
            6 => Ok(Self::LucroArbitrado),
            7 => Ok(Self::RegimeNormal),
            8 => Ok(Self::Imunidade),
            9 => Ok(Self::Isencao),
            other => Err(ApiError::Validation(format!(
                "regime_tributario inválido no banco: {other}"
            ))),
        }
    }
}

impl Serialize for RegimeTributario {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for RegimeTributario {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Self::parse(&raw).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializa_uppercase_underscore() {
        assert_eq!(
            RegimeTributario::SimplesNacionalExcessoSublimite.as_str(),
            "SIMPLES_NACIONAL_EXCESSO_SUBLIMITE"
        );
    }

    #[test]
    fn legado_crt() {
        assert_eq!(
            RegimeTributario::from_legacy_crt(2),
            RegimeTributario::SimplesNacionalExcessoSublimite
        );
    }

    #[test]
    fn db_code_round_trip() {
        for r in RegimeTributario::ALL {
            assert_eq!(
                RegimeTributario::from_db_code(r.as_db_code()).unwrap(),
                r
            );
        }
    }
}
