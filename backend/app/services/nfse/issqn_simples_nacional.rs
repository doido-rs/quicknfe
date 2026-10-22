//! Regras E0621 / E0625 — `pAliq` / `vBC` / `vISSQN` para ME/EPP com apuração ISSQN pelo SN.

use doido::model::sea_orm::prelude::Decimal;

use crate::models::documento_fiscal::Model as Documento;
use crate::services::api_error::ApiError;
use crate::services::nfse::dps_context::DpsContext;
use crate::services::nfse::op_simp_nac::{op_simp_nac_from_regime_code, reg_ap_trib_sn_from_regime_code, OpSimpNac};

/// Alíquota mínima quando ME/EPP + SN + retenção (E0621).
pub fn p_aliq_minima_me_epp_retencao() -> Decimal {
    Decimal::try_from(1.8).unwrap_or_else(|_| Decimal::from(2))
}

/// ME/EPP com ISSQN apurado pelo Simples Nacional (`regApTribSN = 1`).
pub fn me_epp_apura_issqn_pelo_sn(regime_tributario: i32) -> bool {
    let Ok(op) = op_simp_nac_from_regime_code(regime_tributario) else {
        return false;
    };
    if op != OpSimpNac::MeEpp {
        return false;
    }
    matches!(
        reg_ap_trib_sn_from_regime_code(regime_tributario),
        Ok(Some(ref s)) if s == "1"
    )
}

pub fn tp_ret_issqn_com_retencao(doc: &Documento) -> bool {
    doc.issqn_retencao == 1
}

pub fn tem_indicacao_beneficio_municipal(doc: &Documento) -> bool {
    doc.issqn_beneficio_municipal == 1
}

pub fn tem_campos_aliquota_issqn_preenchidos(doc: &Documento) -> bool {
    doc.issqn_aliquota.is_some()
        || doc.issqn_base_calculo.is_some()
        || doc.issqn_valor.is_some()
}

/// E0625: omitir `pAliq` / `vBC` / `vISSQN` no XML (ME/EPP + SN + município ativo + sem BM + sem retenção).
pub fn omitir_campos_aliquota_issqn_me_epp(
    ctx: &DpsContext,
) -> bool {
    me_epp_apura_issqn_pelo_sn(ctx.prestador.regime_tributario)
        && ctx.municipio_incidencia_sn_ativo
        && !tem_indicacao_beneficio_municipal(&ctx.doc)
        && !tp_ret_issqn_com_retencao(&ctx.doc)
}

/// E0621: exige `pAliq` (mín. 1,8%) no cenário ME/EPP + SN + município ativo + retenção.
pub fn exige_aliquota_issqn_me_epp(ctx: &DpsContext) -> bool {
    me_epp_apura_issqn_pelo_sn(ctx.prestador.regime_tributario)
        && ctx.municipio_incidencia_sn_ativo
        && !tem_indicacao_beneficio_municipal(&ctx.doc)
        && tp_ret_issqn_com_retencao(&ctx.doc)
}

/// ME/EPP + SN + município ativo → não emitir BM (E0536).
pub fn proibir_beneficio_municipal_me_epp_sn(ctx: &DpsContext) -> bool {
    me_epp_apura_issqn_pelo_sn(ctx.prestador.regime_tributario)
        && ctx.municipio_incidencia_sn_ativo
}

pub fn limpar_campos_aliquota_issqn(doc: &mut Documento) {
    doc.issqn_aliquota = None;
    doc.issqn_base_calculo = None;
    doc.issqn_valor = None;
}

pub fn validar_e0621_e0625(ctx: &DpsContext) -> Result<(), ApiError> {
    if omitir_campos_aliquota_issqn_me_epp(ctx) && tem_campos_aliquota_issqn_preenchidos(&ctx.doc) {
        return Err(ApiError::Validation(
            "E0625: para prestador ME/EPP com ISSQN apurado pelo Simples Nacional, sem retenção \
             (ISS retido = Não) e município de incidência ativo, não informe alíquota, BC ou valor \
             do ISSQN. Limpe os campos na aba Tributos ou marque ISS retido se houver retenção."
                .into(),
        ));
    }

    if exige_aliquota_issqn_me_epp(ctx) {
        let Some(p) = ctx.doc.issqn_aliquota else {
            return Err(ApiError::Validation(
                "E0621: informe a alíquota ISSQN (%) quando houver retenção para prestador ME/EPP \
                 com apuração pelo Simples Nacional (mínimo 1,8%)."
                    .into(),
            ));
        };
        if p < p_aliq_minima_me_epp_retencao() {
            return Err(ApiError::Validation(format!(
                "E0621: alíquota ISSQN ({p}%) inferior ao mínimo 1,8% para ME/EPP com retenção \
                 e apuração pelo Simples Nacional."
            )));
        }
    }

    if proibir_beneficio_municipal_me_epp_sn(ctx) && tem_indicacao_beneficio_municipal(&ctx.doc) {
        return Err(ApiError::Validation(
            "E0536: prestador ME/EPP apurando ISSQN pelo Simples Nacional não pode informar \
             benefício municipal neste cenário. Desmarque benefício municipal na nota."
                .into(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::regime_tributario::RegimeTributario;
    use crate::services::nfse::rn_validacao::minimal_dps_context;

    #[test]
    fn omitir_quando_me_epp_sem_retencao_municipio_ativo() {
        let mut ctx = minimal_dps_context();
        ctx.prestador.regime_tributario = RegimeTributario::SimplesNacional.as_db_code();
        ctx.municipio_incidencia_sn_ativo = true;
        ctx.doc.issqn_retencao = 0;
        ctx.doc.issqn_beneficio_municipal = 0;
        assert!(omitir_campos_aliquota_issqn_me_epp(&ctx));
    }

    #[test]
    fn exige_aliquota_com_retencao() {
        let mut ctx = minimal_dps_context();
        ctx.prestador.regime_tributario = RegimeTributario::SimplesNacional.as_db_code();
        ctx.municipio_incidencia_sn_ativo = true;
        ctx.doc.issqn_retencao = 1;
        assert!(exige_aliquota_issqn_me_epp(&ctx));
    }

    #[test]
    fn validar_rejeita_aliquota_quando_e0625() {
        let mut ctx = minimal_dps_context();
        ctx.prestador.regime_tributario = RegimeTributario::SimplesNacional.as_db_code();
        ctx.municipio_incidencia_sn_ativo = true;
        ctx.doc.issqn_retencao = 0;
        ctx.doc.issqn_beneficio_municipal = 0;
        ctx.doc.issqn_aliquota = Some(Decimal::try_from(2.0).unwrap());
        let err = validar_e0621_e0625(&ctx).unwrap_err();
        assert!(matches!(err, ApiError::Validation(msg) if msg.contains("E0625")));
    }

    #[test]
    fn validar_rejeita_aliquota_baixa_com_retencao_e0621() {
        let mut ctx = minimal_dps_context();
        ctx.prestador.regime_tributario = RegimeTributario::SimplesNacional.as_db_code();
        ctx.municipio_incidencia_sn_ativo = true;
        ctx.doc.issqn_retencao = 1;
        ctx.doc.issqn_aliquota = Some(Decimal::try_from(1.0).unwrap());
        let err = validar_e0621_e0625(&ctx).unwrap_err();
        assert!(matches!(err, ApiError::Validation(msg) if msg.contains("E0621")));
    }
}
