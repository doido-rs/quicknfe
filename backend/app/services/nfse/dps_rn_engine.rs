//! Motor incremental de RN a partir de `layout_dps.csv` (Obrig. + Rej.).

use doido::model::sea_orm::prelude::Decimal;

use super::dps_context::DpsContext;
use super::dps_rn::{load_rn_dps_rows, RnRow};
use super::endereco_brasil;
use crate::services::api_error::ApiError;
use crate::services::nfse::ambiente::AmbienteNfse;

fn normalize(s: &str) -> String {
    s.trim().to_lowercase()
}

fn is_obrig_rej(row: &RnRow) -> bool {
    normalize(&row.aplic).contains("obrig") && normalize(&row.efeito).starts_with("rej")
}

/// Regras RN simples derivadas do CSV (expandir por fases).
pub fn validar_rn_csv_incremental(ctx: &DpsContext) -> Result<(), ApiError> {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let rows = load_rn_dps_rows(&root).map_err(|e| ApiError::Internal(format!("RN CSV: {e}")))?;

    for row in rows.iter().filter(|r| is_obrig_rej(r)) {
        aplicar_regra_conhecida(ctx, row)?;
    }
    Ok(())
}

fn aplicar_regra_conhecida(ctx: &DpsContext, row: &RnRow) -> Result<(), ApiError> {
    let campo = row.campo.trim();
    if campo.is_empty() {
        return Ok(());
    }

    match campo {
        "tpAmb" if row.cod_erro == "E0006" || row.descricao.contains("Ambiente") => {
            AmbienteNfse::from_codigo(ctx.doc.ambiente)?.ensure_matches_config()?;
        }
        "tpEmit" if row.cod_erro == "E9996" || row.descricao.contains("tomador ou interm") => {}
        "vServ" if campo == "vServ" && is_obrig_rej(row) => {
            if ctx.doc.valor_total_servicos_produtos <= Decimal::ZERO {
                return Err(ApiError::Validation(
                    "RN: valor da operação (vServ) deve ser positivo".into(),
                ));
            }
        }
        "cTribNac" if is_obrig_rej(row) => {
            let ok = ctx.itens.iter().any(|i| {
                i.codigo_tributacao_nacional
                    .as_ref()
                    .is_some_and(|s| !s.trim().is_empty())
            });
            if !ok {
                return Err(ApiError::Validation(
                    "RN: item deve informar código de tributação nacional (cTribNac)".into(),
                ));
            }
        }
        "CEP" | "xLgr" | "nro" | "xBairro" if is_obrig_rej(row) => {
            endereco_brasil::endereco_prestador(
                ctx.prestador.cep.as_deref(),
                ctx.prestador.logradouro.as_deref(),
                ctx.prestador.numero.as_deref(),
                ctx.prestador.bairro.as_deref(),
            )?;
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::nfse::rn_validacao::minimal_dps_context;

    #[test]
    fn rn_engine_aceita_contexto_minimo() {
        validar_rn_csv_incremental(&minimal_dps_context()).expect("ok");
    }
}
