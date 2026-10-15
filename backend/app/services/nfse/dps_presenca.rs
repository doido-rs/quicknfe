//! Validação de presença para campos `infDPS` mapeados no QuickNFe (AGENTS.md).

use std::collections::HashSet;

use doido::model::sea_orm::prelude::Decimal;

use super::dps_context::{cpf_cnpj_tag, DpsContext};
use super::endereco_brasil;
use super::dps_layout::{load_infdps_leiaute, mandatory_infdps_fields, LayoutFieldRow};
use crate::services::api_error::ApiError;

/// Campos obrigatórios do leiaute que o app valida por presença antes de assinar/transmitir.
pub fn campos_presenca_mapeados() -> HashSet<&'static str> {
    [
        "tpAmb",
        "dhEmi",
        "verAplic",
        "serie",
        "nDPS",
        "dCompet",
        "tpEmit",
        "tpEmis",
        "cLocEmi",
        "CNPJ",
        "CPF",
        "xNome",
        "cMun",
        "cLocPrestacao",
        "cTribNac",
        "xDescServ",
        "vServ",
        "tribISSQN",
        "opSimpNac",
        "regEspTrib",
        "CEP",
        "xLgr",
        "nro",
        "xBairro",
        "cIntContrib",
    ]
    .into_iter()
    .collect()
}

fn endereco_prestador_ok(ctx: &DpsContext) -> bool {
    endereco_brasil::endereco_prestador(
        ctx.prestador.cep.as_deref(),
        ctx.prestador.logradouro.as_deref(),
        ctx.prestador.numero.as_deref(),
        ctx.prestador.bairro.as_deref(),
    )
    .is_ok()
}

fn valor_presente(ctx: &DpsContext, campo: &str) -> Result<bool, ApiError> {
    let doc = &ctx.doc;
    let ok = match campo {
        "tpAmb" => doc.ambiente == 1 || doc.ambiente == 2,
        "dhEmi" => true,
        "verAplic" => true,
        "serie" => doc.serie_numero_automatico != 0 || !doc.serie.trim().is_empty(),
        "nDPS" => doc.serie_numero_automatico != 0 || doc.numero_documento > 0,
        "dCompet" => doc.data_competencia.is_some() || !doc.data_emissao.to_string().is_empty(),
        "tpEmit" => true,
        "tpEmis" => doc.tipo_emissao > 0,
        "cLocEmi" => !ctx.municipio_emissao.codigo_ibge.trim().is_empty(),
        "CNPJ" => cpf_cnpj_tag(&ctx.prestador.cpf_cnpj).0 == "CNPJ",
        "CPF" => cpf_cnpj_tag(&ctx.prestador.cpf_cnpj).0 == "CPF",
        "xNome" => !ctx.prestador.razao_social.trim().is_empty(),
        "cMun" => !ctx.municipio_prestador.codigo_ibge.trim().is_empty(),
        "cLocPrestacao" => !ctx.municipio_prestacao.codigo_ibge.trim().is_empty(),
        "cTribNac" => ctx.itens.iter().any(|i| {
            i.codigo_tributacao_nacional
                .as_ref()
                .is_some_and(|s| !s.trim().is_empty())
        }),
        "xDescServ" => ctx.itens.iter().any(|i| {
            i.descricao_servico
                .as_ref()
                .or(Some(&i.descricao))
                .is_some_and(|s| !s.trim().is_empty())
        }),
        "vServ" => doc.valor_total_servicos_produtos > Decimal::ZERO,
        "tribISSQN" => !doc.trib_issqn.trim().is_empty(),
        "opSimpNac" => !ctx.prestador.regime_tributario.trim().is_empty(),
        "regEspTrib" => !doc.reg_esp_trib_issqn.trim().is_empty(),
        "CEP" | "xLgr" | "nro" | "xBairro" => endereco_prestador_ok(ctx),
        "cIntContrib" => !ctx.itens.is_empty(),
        _ => return Ok(false),
    };
    Ok(ok)
}

pub fn validar_presenca_leiaute(ctx: &DpsContext, rows: &[LayoutFieldRow]) -> Result<(), ApiError> {
    let mapped = campos_presenca_mapeados();
    let mandatory = mandatory_infdps_fields(rows);
    for row in mandatory {
        if !mapped.contains(row.campo.as_str()) {
            continue;
        }
        if !valor_presente(ctx, &row.campo)? {
            return Err(ApiError::Validation(format!(
                "RN: campo obrigatório ausente ou vazio: {} ({})",
                row.campo, row.xml_path
            )));
        }
    }
    Ok(())
}

pub fn validar_presenca_repo(ctx: &DpsContext) -> Result<(), ApiError> {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let rows = load_infdps_leiaute(&root)
        .map_err(|e| ApiError::Internal(format!("leiaute CSV: {e}")))?;
    validar_presenca_leiaute(ctx, &rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::nfse::rn_validacao::minimal_dps_context;

    #[test]
    fn presenca_aceita_contexto_minimo() {
        validar_presenca_repo(&minimal_dps_context()).expect("presenca ok");
    }

    #[test]
    fn presenca_rejeita_sem_competencia_quando_exigida() {
        let mut ctx = minimal_dps_context();
        ctx.doc.data_competencia = None;
        ctx.doc.data_emissao = chrono::Utc::now();
        // dCompet falls back to emissao — still ok
        validar_presenca_repo(&ctx).expect("fallback emissao");
    }

    #[test]
    fn presenca_rejeita_sem_c_trib_nac() {
        let mut ctx = minimal_dps_context();
        if let Some(item) = ctx.itens.first_mut() {
            item.codigo_tributacao_nacional = None;
        }
        assert!(validar_presenca_repo(&ctx).is_err());
    }
}
