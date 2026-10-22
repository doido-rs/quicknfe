use doido::model::sea_orm::prelude::Decimal;
use serde::Deserialize;

use crate::models::cliente::Model as Cliente;
use crate::models::localidade_parte::LocalidadeParte;
use crate::models::regime_tributario::RegimeTributario;
use crate::services::nfse::dps_context::DpsContext;
use crate::services::nfse::dps_rn_catalog::rule_by_cod;
use crate::services::nfse::rn::dispatch_generated::evaluate_cod;
use crate::services::nfse::rn_validacao::minimal_dps_context;

#[derive(Debug, Deserialize)]
struct RuleCase {
    cod_erro: String,
    mutate: serde_json::Map<String, serde_json::Value>,
    pending: bool,
}

fn apply_mutate(ctx: &mut DpsContext, mutate: &serde_json::Map<String, serde_json::Value>) {
    for (key, value) in mutate {
        match key.as_str() {
            "valor_total_servicos_produtos" => {
                ctx.doc.valor_total_servicos_produtos =
                    Decimal::from(value.as_str().and_then(|s| s.parse().ok()).unwrap_or(0));
            }
            "clear_c_trib_nac" => {
                if let Some(item) = ctx.itens.first_mut() {
                    item.codigo_tributacao_nacional = None;
                }
            }
            "codigo_tributacao_municipal" => {
                if let Some(item) = ctx.itens.first_mut() {
                    item.codigo_tributacao_municipal = value.as_str().map(String::from);
                }
            }
            "tomador_same_as_prestador" => {
                ctx.doc.tomador_localidade = LocalidadeParte::Brasil.as_db_code();
                ctx.doc.cliente_id = Some(99);
                ctx.tomador = Some(Cliente {
                    id: 99,
                    empresa_id: 1,
                    cpf_cnpj: ctx.prestador.cpf_cnpj.clone(),
                    razao_social: "Tomador".into(),
                    email: None,
                    telefone: None,
                    inscricao_municipal: None,
                    inscricao_estadual: None,
                    municipio_id: Some(1),
                    cep: None,
                    logradouro: None,
                    numero: None,
                    bairro: None,
                });
            }
            "prestador_regime_mismatch_cnc" => {
                ctx.prestador.regime_tributario = RegimeTributario::SimplesNacional.as_db_code();
                ctx.prestador.op_simp_nac_verificado = Some(1);
            }
            _ => {}
        }
    }
}

#[test]
fn rn_cases_sem_pending() {
    let raw = include_str!("rn_cases.json");
    let cases: Vec<RuleCase> = serde_json::from_str(raw).expect("cases json");
    let pending: Vec<_> = cases.iter().filter(|c| c.pending).collect();
    assert!(
        pending.is_empty(),
        "casos pendentes: {}",
        pending.len()
    );
}

#[test]
fn rn_cases_disparam_codigo() {
    let raw = include_str!("rn_cases.json");
    let cases: Vec<RuleCase> = serde_json::from_str(raw).expect("cases json");
    for case in cases {
        if case.pending {
            continue;
        }
        let mut ctx = minimal_dps_context();
        if case.mutate.is_empty() {
            evaluate_cod(&case.cod_erro, &ctx).expect(&format!("ok minimal {}", case.cod_erro));
            continue;
        }
        apply_mutate(&mut ctx, &case.mutate);
        if rule_by_cod(&case.cod_erro)
            .is_some_and(|m| m.requires_db_catalog)
        {
            continue;
        }
        let err = evaluate_cod(&case.cod_erro, &ctx)
            .expect_err(&format!("deve falhar {}", case.cod_erro));
        assert_eq!(err.cod_erro, case.cod_erro);
    }
}
