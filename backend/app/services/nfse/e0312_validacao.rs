//! Validação E0312 — CNA administrado pelo município de incidência na competência.

use chrono::NaiveDate;
use doido::model::sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};

use crate::models::cna::{Column as CnaColumn, Entity as CnaEntity};
use crate::models::municipio_cna_parametrizacao::{Column as ParamColumn, Entity as ParamEntity};
use crate::services::api_error::ApiError;
use crate::services::nfse::codigo_servico::normalize_c_trib_nac;
use crate::services::nfse::dps_context::DpsContext;
use crate::services::nfse::incidencia_issqn;
use crate::services::nfse::municipio_cna_sync;

pub async fn validar_e0312_catalogo(
    db: &impl ConnectionTrait,
    ctx: &DpsContext,
) -> Result<(), ApiError> {
    let competencia = ctx
        .doc
        .data_competencia
        .ok_or_else(|| ApiError::Validation("RN: data de competência é obrigatória".into()))?;

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

        let cna = CnaEntity::find()
            .filter(CnaColumn::Codigo.eq(&codigo))
            .one(db)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?
            .ok_or_else(|| {
                ApiError::Validation(format!(
                    "Código de tributação nacional {codigo} não consta no catálogo sincronizado"
                ))
            })?;

        let ibge = incidencia_issqn::codigo_ibge_incidencia(ctx, &cna)?;
        let municipio = incidencia_issqn::resolve_municipio_by_ibge(&ibge, ctx).ok_or_else(|| {
            ApiError::Validation(format!(
                "Município de incidência IBGE {ibge} não encontrado no contexto da DPS"
            ))
        })?;

        if !municipio_cna_sync::municipio_tem_parametrizacao(db, municipio.id)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?
        {
            return Err(ApiError::Validation(format!(
                "Parametrização municipal desatualizada para {} (IBGE {}). \
                 Aguarde a carga diária do catálogo NFS-e ou contate o administrador.",
                municipio.nome, ibge
            )));
        }

        let rows = ParamEntity::find()
            .filter(ParamColumn::MunicipioId.eq(municipio.id))
            .filter(ParamColumn::CodigoTribNac.eq(&codigo))
            .all(db)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;

        let vigente: Vec<_> = rows
            .iter()
            .filter(|r| vigente_em(competencia, r.vigencia_inicio, r.vigencia_fim))
            .collect();

        if vigente.is_empty() {
            return Err(ApiError::Validation(format!(
                "E0312: o código {codigo} não está administrado pelo município de incidência \
                 {} (IBGE {}) na competência {competencia}. \
                 Ajuste o CNA do item ou a localidade da nota.",
                municipio.nome, ibge
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
