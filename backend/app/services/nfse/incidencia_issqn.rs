//! Cálculo do município de incidência do ISSQN (LI) a partir do CNA e do contexto DPS.

use crate::models::cna::Model as Cna;
use crate::models::municipio::Model as Municipio;
use crate::services::api_error::ApiError;
use crate::services::nfse::dps_context::DpsContext;

/// IBGE (7 dígitos) do município de incidência conforme flags EP/LP/ET do CNA.
pub fn codigo_ibge_incidencia(ctx: &DpsContext, cna: &Cna) -> Result<String, ApiError> {
    if cna.incid_lp != 0 {
        return Ok(pad_ibge7(&ctx.municipio_prestacao.codigo_ibge));
    }
    if cna.incid_et != 0 {
        let mun = ctx
            .tomador_municipio
            .as_ref()
            .ok_or_else(|| {
                ApiError::Validation(
                    "RN: incidência no tomador exige tomador com município cadastrado".into(),
                )
            })?;
        return Ok(pad_ibge7(&mun.codigo_ibge));
    }
    if cna.incid_ep != 0 {
        return Ok(pad_ibge7(&ctx.municipio_prestador.codigo_ibge));
    }
    // Fallback: prestador (regra geral LC 116 / serviços de TI).
    Ok(pad_ibge7(&ctx.municipio_prestador.codigo_ibge))
}

fn pad_ibge7(raw: &str) -> String {
    let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() >= 7 {
        digits[digits.len() - 7..].to_string()
    } else {
        format!("{:0>7}", digits)
    }
}

pub fn resolve_municipio_by_ibge<'a>(
    ibge: &str,
    ctx: &'a DpsContext,
) -> Option<&'a Municipio> {
    let target = pad_ibge7(ibge);
    for m in [
        &ctx.municipio_prestador,
        &ctx.municipio_prestacao,
        &ctx.municipio_emissao,
    ] {
        if pad_ibge7(&m.codigo_ibge) == target {
            return Some(m);
        }
    }
    ctx.tomador_municipio
        .as_ref()
        .filter(|m| pad_ibge7(&m.codigo_ibge) == target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::nfse::rn_validacao::minimal_dps_context;

    #[test]
    fn incidencia_ep_usa_prestador() {
        let ctx = minimal_dps_context();
        let cna = Cna {
            id: 1,
            codigo: "010101".into(),
            descricao: "Teste".into(),
            incid_ep: 1,
            incid_lp: 0,
            incid_et: 0,
            grupo_obra: 0,
            grupo_atv_evento: 0,
            synced_at: None,
        };
        let ibge = codigo_ibge_incidencia(&ctx, &cna).expect("ok");
        assert_eq!(ibge, "3304557");
    }
}
