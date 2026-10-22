//! Incidência ISSQN do subitem (CNA) conforme parametrização municipal na competência.

use chrono::NaiveDate;
use doido::model::sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};

use crate::models::cna::{Column as CnaColumn, Entity as CnaEntity, Model as Cna};
use crate::models::item_documento_fiscal::Model as ItemDoc;
use crate::models::municipio::Model as Municipio;
use crate::models::municipio_cna_parametrizacao::{Column as ParamColumn, Entity as ParamEntity};
use crate::services::api_error::ApiError;
use crate::services::nfse::codigo_servico::{normalize_c_trib_mun, normalize_c_trib_nac};
use crate::services::nfse::dps_context::DpsContext;
use crate::services::nfse::e0312_validacao::vigente_em;
use crate::services::nfse::incidencia_issqn;
use crate::services::nfse::municipio_cna_sync;

/// `cTribNac` 99.01.01 — sem incidência de ISSQN/ICMS (layout / E0532).
pub fn codigo_nacional_sem_incidencia_issqn(codigo_nacional: &str) -> bool {
    normalize_c_trib_nac(codigo_nacional).as_deref() == Some("990101")
}

/// Subitem incidente no município de incidência quando há parametrização vigente na competência.
pub async fn subitem_incidente_municipio_incidencia(
    db: &impl ConnectionTrait,
    ctx: &DpsContext,
    item: &ItemDoc,
    competencia: NaiveDate,
) -> Result<bool, ApiError> {
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

    if codigo_nacional_sem_incidencia_issqn(&codigo) {
        return Ok(false);
    }

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

    let municipio = resolve_municipio_incidencia(ctx, &cna)?;

    if !municipio_cna_sync::municipio_tem_parametrizacao(db, municipio.id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
    {
        return Err(ApiError::Validation(format!(
            "Parametrização municipal desatualizada para {} (IBGE {}). \
             Aguarde a carga diária do catálogo NFS-e ou contate o administrador.",
            municipio.nome,
            pad_ibge(&municipio.codigo_ibge)
        )));
    }

    let codigo_mun = normalize_c_trib_mun(
        item.codigo_tributacao_municipal
            .as_deref()
            .unwrap_or("000"),
    );

    let rows = ParamEntity::find()
        .filter(ParamColumn::MunicipioId.eq(municipio.id))
        .filter(ParamColumn::CodigoTribNac.eq(&codigo))
        .all(db)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let vigente: Vec<_> = rows
        .iter()
        .filter(|r| vigente_em(competencia, r.vigencia_inicio, r.vigencia_fim))
        .filter(|r| r.codigo_trib_mun == codigo_mun || codigo_mun == "000")
        .collect();

    if !vigente.is_empty() {
        return Ok(true);
    }

    let vigente_qualquer_desdobro: Vec<_> = rows
        .iter()
        .filter(|r| vigente_em(competencia, r.vigencia_inicio, r.vigencia_fim))
        .collect();

    Ok(!vigente_qualquer_desdobro.is_empty())
}

/// Município de incidência ISSQN (primeiro item com CNA ou prestador).
pub async fn municipio_incidencia_id(
    db: &impl ConnectionTrait,
    ctx: &DpsContext,
) -> Result<i64, ApiError> {
    if let Some(item) = ctx.itens.first() {
        let raw = item
            .codigo_tributacao_nacional
            .as_deref()
            .filter(|s| !s.trim().is_empty());
        if let Some(raw) = raw {
            if let Some(codigo) = crate::services::nfse::codigo_servico::normalize_c_trib_nac(raw)
            {
                if let Some(cna) = CnaEntity::find()
                    .filter(CnaColumn::Codigo.eq(&codigo))
                    .one(db)
                    .await
                    .map_err(|e| ApiError::Internal(e.to_string()))?
                {
                    return Ok(resolve_municipio_incidencia(ctx, &cna)?.id);
                }
            }
        }
    }
    Ok(ctx.municipio_prestador.id)
}

pub async fn municipio_incidencia_sn_ativo(
    db: &impl ConnectionTrait,
    ctx: &DpsContext,
) -> Result<bool, ApiError> {
    let municipio_id = municipio_incidencia_id(db, ctx).await?;
    municipio_cna_sync::municipio_tem_parametrizacao(db, municipio_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))
}

fn resolve_municipio_incidencia(ctx: &DpsContext, cna: &Cna) -> Result<Municipio, ApiError> {
    let ibge = incidencia_issqn::codigo_ibge_incidencia(ctx, cna)?;
    let municipio = incidencia_issqn::resolve_municipio_by_ibge(&ibge, ctx).ok_or_else(|| {
        ApiError::Validation(format!(
            "Município de incidência IBGE {ibge} não encontrado no contexto da DPS"
        ))
    })?;
    Ok(municipio.clone())
}

fn pad_ibge(raw: &str) -> String {
    let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() >= 7 {
        digits[digits.len() - 7..].to_string()
    } else {
        format!("{:0>7}", digits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detecta_990101_sem_incidencia() {
        assert!(codigo_nacional_sem_incidencia_issqn("990101"));
        assert!(codigo_nacional_sem_incidencia_issqn("99.01.01"));
        assert!(!codigo_nacional_sem_incidencia_issqn("010101"));
    }
}
