//! E0532 / E0539 / E0540 — `tribISSQN` vs incidência municipal dos itens (parametrização ADN).

use doido::model::sea_orm::ConnectionTrait;

use crate::models::trib_issqn::TribIssqn;
use crate::services::api_error::ApiError;
use crate::services::entity_enums::trib_issqn_from_db;
use crate::services::nfse::catalog_item_incidencia::{
    codigo_nacional_sem_incidencia_issqn, subitem_incidente_municipio_incidencia,
};
use crate::services::nfse::codigo_servico::normalize_c_trib_nac;
use crate::services::nfse::dps_context::DpsContext;

pub async fn validar_trib_issqn_incidencia_catalogo(
    db: &impl ConnectionTrait,
    ctx: &DpsContext,
) -> Result<(), ApiError> {
    let competencia = ctx
        .doc
        .data_competencia
        .ok_or_else(|| ApiError::Validation("RN: data de competência é obrigatória".into()))?;

    let trib = trib_issqn_from_db(ctx.doc.trib_issqn)?;

    for item in &ctx.itens {
        let raw = item
            .codigo_tributacao_nacional
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| {
                ApiError::Validation("RN: item deve informar código de tributação nacional".into())
            })?;
        let codigo = normalize_c_trib_nac(raw).ok_or_else(|| {
            ApiError::Validation(format!("Código de tributação nacional inválido: {raw}"))
        })?;

        if codigo_nacional_sem_incidencia_issqn(&codigo) && trib != TribIssqn::NaoIncidencia {
            return Err(ApiError::Validation(
                "E0532: o serviço 99.01.01 exige tributação ISSQN \"4 - Não Incidência\" \
                 (tribISSQN = 4)."
                    .into(),
            ));
        }

        let incidente =
            subitem_incidente_municipio_incidencia(db, ctx, item, competencia).await?;

        if trib == TribIssqn::NaoIncidencia && incidente {
            let mun = item
                .codigo_tributacao_municipal
                .as_deref()
                .unwrap_or("000");
            return Err(ApiError::Validation(format!(
                "E0539: não é permitido tribISSQN = 4 (Não Incidência) para o código \
                 {codigo} (cTribMun {mun}): o subitem é incidente na parametrização do \
                 município de incidência na competência {competencia}. \
                 Altere a tributação ISSQN na aba Tributos ou ajuste CNA/código municipal do item."
            )));
        }

        if trib != TribIssqn::NaoIncidencia && !incidente {
            return Err(ApiError::Validation(format!(
                "E0540: o código {codigo} não é incidente no município de incidência na \
                 competência {competencia}; use tribISSQN = 4 (Não Incidência) ou revise o item."
            )));
        }
    }

    Ok(())
}
