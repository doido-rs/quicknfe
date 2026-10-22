//! Lógica compartilhada das regras RN (uma função por `cod_erro` via dispatch gerado).

use doido::model::sea_orm::prelude::Decimal;

use crate::models::localidade_parte::LocalidadeParte;
use crate::services::entity_enums::localidade_from_db;
use crate::services::nfse::ambiente::AmbienteNfse;
use crate::services::nfse::dps_context::{cpf_cnpj_tag, only_digits, DpsContext};
use crate::services::nfse::dps_rn_catalog::{rule_by_cod, RuleGroup};
use crate::services::nfse::endereco_brasil;
use crate::services::nfse::op_simp_nac::validar_e0160_op_simp_nac;
use crate::services::nfse::rn::violations::{violation_for_cod, RnViolation};

fn skip_at_dps_save(cod: &str) -> bool {
    let Some(meta) = rule_by_cod(cod) else {
        return false;
    };
    matches!(
        meta.group,
        RuleGroup::Ibscbs
            | RuleGroup::Assinatura
            | RuleGroup::DeducaoReducao
            | RuleGroup::Exportacao
            | RuleGroup::Evento
            | RuleGroup::Substituicao
    )
}

pub fn evaluate_rule(cod: &str, ctx: &DpsContext) -> Result<(), RnViolation> {
    if skip_at_dps_save(cod) {
        return Ok(());
    }

    match cod {
        "E0001" | "E0004" => check_valor_operacao_positivo(ctx, cod),
        "E0006" => check_tp_amb(ctx),
        "E0160" => check_e0160(ctx),
        "E0202" => check_prestador_diferente_tomador(ctx),
        "E0315" => check_c_trib_mun_invalido(ctx),
        "E1260" => Ok(()),
        _ => check_presence_and_catalog_rules(cod, ctx),
    }
}

fn check_valor_operacao_positivo(ctx: &DpsContext, cod: &str) -> Result<(), RnViolation> {
    if ctx.doc.valor_total_servicos_produtos <= Decimal::ZERO {
        return Err(violation_for_cod(
            cod,
            vec!["valor_total_servicos_produtos".into()],
        ));
    }
    Ok(())
}

fn check_tp_amb(ctx: &DpsContext) -> Result<(), RnViolation> {
    if let Err(e) = AmbienteNfse::from_codigo(ctx.doc.ambiente).and_then(|a| a.ensure_matches_config())
    {
        return Err(RnViolation::with_cod(
            "E0006",
            e.to_string(),
            vec!["ambiente".into()],
        ));
    }
    Ok(())
}

fn check_e0160(ctx: &DpsContext) -> Result<(), RnViolation> {
    validar_e0160_op_simp_nac(ctx).map_err(|e| {
        RnViolation::with_cod("E0160", e.to_string(), vec!["empresa_id".into()])
    })
}

fn check_prestador_diferente_tomador(ctx: &DpsContext) -> Result<(), RnViolation> {
    if localidade_from_db(ctx.doc.tomador_localidade).ok() != Some(LocalidadeParte::Brasil) {
        return Ok(());
    }
    let Some(t) = &ctx.tomador else {
        return Ok(());
    };
    let prest = only_digits(&ctx.prestador.cpf_cnpj);
    let toma = only_digits(&t.cpf_cnpj);
    if prest.len() == 14 && prest == toma {
        return Err(violation_for_cod("E0202", vec!["cliente_id".into()]));
    }
    Ok(())
}

fn check_c_trib_mun_invalido(ctx: &DpsContext) -> Result<(), RnViolation> {
    let bad = ctx.itens.iter().any(|i| {
        i.codigo_tributacao_municipal
            .as_ref()
            .map(|s| s.trim())
            .is_some_and(|s| s.is_empty() || s == "000")
    });
    if bad {
        return Err(violation_for_cod(
            "E0315",
            vec!["itens.codigo_tributacao_municipal".into()],
        ));
    }
    Ok(())
}

fn check_presence_and_catalog_rules(cod: &str, ctx: &DpsContext) -> Result<(), RnViolation> {
    let Some(meta) = rule_by_cod(cod) else {
        return Ok(());
    };

    match meta.campo {
        "cTribNac" if meta.group == RuleGroup::ServicoItem => {
            let ok = ctx.itens.iter().any(|i| {
                i.codigo_tributacao_nacional
                    .as_ref()
                    .is_some_and(|s| !s.trim().is_empty())
            });
            if !ok {
                return Err(violation_for_cod(cod, vec!["itens".into()]));
            }
        }
        "CEP" | "xLgr" | "nro" | "xBairro" if meta.group == RuleGroup::Endereco => {
            endereco_brasil::endereco_prestador(
                ctx.prestador.cep.as_deref(),
                ctx.prestador.logradouro.as_deref(),
                ctx.prestador.numero.as_deref(),
                ctx.prestador.bairro.as_deref(),
            )
            .map_err(|e| RnViolation::with_cod(cod, e.to_string(), vec!["empresa_id".into()]))?;
        }
        "CNPJ" if meta.group == RuleGroup::Prestador => {
            if cpf_cnpj_tag(&ctx.prestador.cpf_cnpj).0 != "CNPJ" {
                return Err(violation_for_cod(cod, vec!["empresa_id".into()]));
            }
        }
        "vServ" => return check_valor_operacao_positivo(ctx, cod),
        _ => {
            if meta.group == RuleGroup::CatalogoMunicipal || meta.group == RuleGroup::RegraGeral {
                return Ok(());
            }
        }
    }
    Ok(())
}
