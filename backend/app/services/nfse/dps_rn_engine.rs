//! Motor incremental de RN a partir de `layout_dps.csv` (Obrig. + Rej.).

use super::dps_context::DpsContext;
use super::dps_rn::{load_rn_dps_rows, RnRow};
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
        "tpEmit" if row.cod_erro == "E9996" || row.descricao.contains("tomador ou interm") => {
            // QuickNFe emite apenas tpEmit=1 (prestador).
            if ctx.doc.tipo_emissao != 1 {
                // tipo_emissao no doc; tpEmit fixo 1 no XML — nada a fazer
            }
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
