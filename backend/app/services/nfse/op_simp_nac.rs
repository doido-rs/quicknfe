//! Mapeamento `regime_tributario` ↔ `opSimpNac` / `regApTribSN` (RN E0160, E0166).

use crate::models::regime_tributario::RegimeTributario;
use crate::services::api_error::ApiError;
use crate::services::entity_enums::regime_from_db;
use crate::services::nfse::dps_context::DpsContext;

/// Valores do leiaute DPS (`opSimpNac`): 1 não optante, 2 MEI, 3 ME/EPP.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpSimpNac {
    NaoOptante = 1,
    Mei = 2,
    MeEpp = 3,
}

impl OpSimpNac {
    pub fn as_layout_str(self) -> &'static str {
        match self {
            Self::NaoOptante => "1",
            Self::Mei => "2",
            Self::MeEpp => "3",
        }
    }

    pub fn as_i32(self) -> i32 {
        self as i32
    }

    pub fn label_pt(self) -> &'static str {
        match self {
            Self::NaoOptante => "Não optante pelo Simples Nacional",
            Self::Mei => "MEI",
            Self::MeEpp => "ME/EPP optante pelo Simples Nacional",
        }
    }
}

pub fn op_simp_nac_from_regime_code(regime_code: i32) -> Result<OpSimpNac, ApiError> {
    let regime = regime_from_db(regime_code)?;
    Ok(op_simp_nac_from_regime(regime))
}

pub fn op_simp_nac_from_regime(regime: RegimeTributario) -> OpSimpNac {
    match regime {
        RegimeTributario::MicroempreendedorIndividual => OpSimpNac::Mei,
        RegimeTributario::SimplesNacional | RegimeTributario::SimplesNacionalExcessoSublimite => {
            OpSimpNac::MeEpp
        }
        _ => OpSimpNac::NaoOptante,
    }
}

/// Regime de apuração SN (obrigatório quando `opSimpNac` = 3 — RN E0166).
pub fn reg_ap_trib_sn_from_regime_code(regime_code: i32) -> Result<Option<String>, ApiError> {
    let regime = regime_from_db(regime_code)?;
    Ok(reg_ap_trib_sn_from_regime(regime))
}

pub fn reg_ap_trib_sn_from_regime(regime: RegimeTributario) -> Option<String> {
    match op_simp_nac_from_regime(regime) {
        OpSimpNac::MeEpp => Some(match regime {
            RegimeTributario::SimplesNacionalExcessoSublimite => "2".into(),
            RegimeTributario::SimplesNacional => "1".into(),
            _ => "1".into(),
        }),
        _ => None,
    }
}

/// Regime tributário sugerido quando o cadastro (CNC/Simples) indica `opSimpNac`.
pub fn regime_sugerido_para_op_simp_nac(op: OpSimpNac) -> RegimeTributario {
    match op {
        OpSimpNac::Mei => RegimeTributario::MicroempreendedorIndividual,
        OpSimpNac::MeEpp => RegimeTributario::SimplesNacional,
        OpSimpNac::NaoOptante => RegimeTributario::RegimeNormal,
    }
}

/// Interpreta `OpcaoSimplesNacional` retornado pelo CNC (0/NENHUM ou 1–3 alinhados ao leiaute).
pub fn parse_opcao_simples_cnc(raw: &str) -> Option<OpSimpNac> {
    let s = raw.trim();
    if s.is_empty() {
        return None;
    }
    if let Ok(n) = s.parse::<i32>() {
        return op_simp_nac_from_cnc_int(n);
    }
    let u = s.to_ascii_uppercase();
    match u.as_str() {
        "NENHUM" | "NAO_OPTANTE" | "NAO OPTANTE" | "NÃO OPTANTE" | "NAOPTANTE" => {
            Some(OpSimpNac::NaoOptante)
        }
        "MEI" => Some(OpSimpNac::Mei),
        "ME_EPP" | "MEEPP" | "ME/EPP" => Some(OpSimpNac::MeEpp),
        _ => None,
    }
}

pub fn op_simp_nac_from_cnc_int(n: i32) -> Option<OpSimpNac> {
    match n {
        0 => Some(OpSimpNac::NaoOptante),
        1 => Some(OpSimpNac::NaoOptante),
        2 => Some(OpSimpNac::Mei),
        3 => Some(OpSimpNac::MeEpp),
        _ => None,
    }
}

pub fn op_simp_nac_from_cnc_json(value: &serde_json::Value) -> Option<OpSimpNac> {
    match value {
        serde_json::Value::Number(n) => n
            .as_i64()
            .and_then(|i| op_simp_nac_from_cnc_int(i as i32)),
        serde_json::Value::String(s) => parse_opcao_simples_cnc(s),
        _ => None,
    }
}

/// `true` quando o regime cadastrado no prestador não gera o mesmo `opSimpNac` do CNC verificado.
pub fn regime_requer_atualizacao_para_cnc(
    regime_code: i32,
    op_simp_cnc: i32,
) -> Result<bool, ApiError> {
    let dps = op_simp_nac_from_regime_code(regime_code)?;
    Ok(dps.as_i32() != op_simp_cnc)
}

/// RN E0160 — valida quando há snapshot CNC (`op_simp_nac_verificado` no prestador).
pub fn validar_e0160_op_simp_nac(ctx: &DpsContext) -> Result<(), ApiError> {
    let Some(verificado) = ctx.prestador.op_simp_nac_verificado else {
        return Ok(());
    };
    let dps = op_simp_nac_from_regime_code(ctx.prestador.regime_tributario)?;
    if dps.as_i32() == verificado {
        return Ok(());
    }
    let cnc_op = OpSimpNac::from_layout_i32(verificado).unwrap_or(OpSimpNac::NaoOptante);
    let sug = regime_sugerido_para_op_simp_nac(cnc_op);
    Err(ApiError::Validation(format!(
        "E0160: opSimpNac na DPS ({}) não confere com o cadastro Simples/CNC consultado ({}). \
         Atualize o regime tributário do prestador para \"{}\" ({}) ou consulte o CNC novamente.",
        dps.as_layout_str(),
        verificado,
        sug.label_pt(),
        sug.as_str()
    )))
}

impl OpSimpNac {
    fn from_layout_i32(n: i32) -> Option<Self> {
        match n {
            1 => Some(Self::NaoOptante),
            2 => Some(Self::Mei),
            3 => Some(Self::MeEpp),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::regime_tributario::RegimeTributario;

    #[test]
    fn regime_para_op_simp_nac() {
        assert_eq!(
            op_simp_nac_from_regime(RegimeTributario::MicroempreendedorIndividual),
            OpSimpNac::Mei
        );
        assert_eq!(
            op_simp_nac_from_regime(RegimeTributario::SimplesNacional),
            OpSimpNac::MeEpp
        );
        assert_eq!(
            op_simp_nac_from_regime(RegimeTributario::RegimeNormal),
            OpSimpNac::NaoOptante
        );
    }

    #[test]
    fn cnc_opcao_zero_e_nenhum() {
        assert_eq!(op_simp_nac_from_cnc_int(0), Some(OpSimpNac::NaoOptante));
        assert_eq!(
            parse_opcao_simples_cnc("NENHUM"),
            Some(OpSimpNac::NaoOptante)
        );
        assert_eq!(parse_opcao_simples_cnc("3"), Some(OpSimpNac::MeEpp));
    }

    #[test]
    fn reg_ap_trib_sn_excesso() {
        assert_eq!(
            reg_ap_trib_sn_from_regime(RegimeTributario::SimplesNacionalExcessoSublimite),
            Some("2".into())
        );
    }
}
