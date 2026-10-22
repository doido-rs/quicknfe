//! Validação E0312 — CNA administrado pelo município de incidência na competência.

use chrono::NaiveDate;
use doido::model::sea_orm::ConnectionTrait;

use crate::models::trib_issqn::TribIssqn;
use crate::services::api_error::ApiError;
use crate::services::entity_enums::trib_issqn_from_db;
use crate::services::nfse::catalog_item_incidencia::{
    codigo_nacional_sem_incidencia_issqn, subitem_incidente_municipio_incidencia,
};
use crate::services::nfse::codigo_servico::normalize_c_trib_nac;
use crate::services::nfse::dps_context::DpsContext;

pub async fn validar_e0312_catalogo(
    db: &impl ConnectionTrait,
    ctx: &DpsContext,
) -> Result<(), ApiError> {
    let competencia = ctx
        .doc
        .data_competencia
        .ok_or_else(|| ApiError::Validation("RN: data de competência é obrigatória".into()))?;

    let trib = trib_issqn_from_db(ctx.doc.trib_issqn)?;

    for item in &ctx.itens {
        let incidente =
            subitem_incidente_municipio_incidencia(db, ctx, item, competencia).await?;

        if trib == TribIssqn::NaoIncidencia {
            let raw = item
                .codigo_tributacao_nacional
                .as_deref()
                .filter(|s| !s.trim().is_empty())
                .ok_or_else(|| {
                    ApiError::Validation(
                        "RN: item deve informar código de tributação nacional".into(),
                    )
                })?;
            let codigo = normalize_c_trib_nac(raw).ok_or_else(|| {
                ApiError::Validation(format!("Código de tributação nacional inválido: {raw}"))
            })?;
            if codigo_nacional_sem_incidencia_issqn(&codigo) || !incidente {
                continue;
            }
        }

        if !incidente {
            let raw = item
                .codigo_tributacao_nacional
                .as_deref()
                .unwrap_or("");
            let codigo = normalize_c_trib_nac(raw).unwrap_or_default();
            return Err(ApiError::Validation(format!(
                "E0312: o código {codigo} não está administrado pelo município de incidência \
                 na competência {competencia}. \
                 Ajuste o CNA do item ou a localidade da nota."
            )));
        }
    }
    Ok(())
}

pub fn vigente_em(
    competencia: NaiveDate,
    inicio: Option<NaiveDate>,
    fim: Option<NaiveDate>,
) -> bool {
    if let Some(i) = inicio {
        if competencia < i {
            return false;
        }
    }
    if let Some(f) = fim {
        if competencia > f {
            return false;
        }
    }
    true
}
