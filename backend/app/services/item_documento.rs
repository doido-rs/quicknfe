//! Normalização de itens da nota (CNA, cTribMun, conta fiscal).

use crate::services::api_error::ApiError;
use crate::services::cna::require_cna;
use crate::services::conta_fiscal::normalize_municipal_code;
use crate::services::nfse::normalize_c_trib_nac;
use crate::services::tenant_scope;

#[derive(Debug, Clone)]
pub struct ItemTributacaoResolved {
    pub cna_id: Option<i64>,
    pub codigo_tributacao_nacional: Option<String>,
    pub codigo_tributacao_municipal: Option<String>,
    pub conta_fiscal_id: Option<i64>,
}

pub async fn resolve_item_tributacao(
    db: &doido::model::sea_orm::DatabaseConnection,
    empresa_id: i64,
    cna_id: Option<i64>,
    codigo_tributacao_municipal: Option<&str>,
    conta_fiscal_id: Option<i64>,
) -> Result<ItemTributacaoResolved, ApiError> {
    if let Some(conta_id) = conta_fiscal_id {
        let conta = tenant_scope::conta_fiscal_for_tenant(db, empresa_id, conta_id).await?;
        let municipal = normalize_municipal_code(&conta.codigo_tributacao_municipal)?;
        return Ok(ItemTributacaoResolved {
            cna_id: Some(conta.cna_id),
            codigo_tributacao_nacional: Some(conta.codigo_tributacao_nacional),
            codigo_tributacao_municipal: Some(municipal),
            conta_fiscal_id: Some(conta_id),
        });
    }

    let Some(cna_id) = cna_id else {
        return Ok(ItemTributacaoResolved {
            cna_id: None,
            codigo_tributacao_nacional: None,
            codigo_tributacao_municipal: None,
            conta_fiscal_id: None,
        });
    };

    if cna_id <= 0 {
        return Err(ApiError::Validation(
            "Código CNA da operação (cna_id) é obrigatório no item".into(),
        ));
    }

    let cna = require_cna(db, cna_id).await?;
    let nacional = normalize_c_trib_nac(&cna.codigo).ok_or_else(|| {
        ApiError::Validation(format!("CNA {} com código inválido", cna.id))
    })?;

    let municipal_raw = codigo_tributacao_municipal
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| {
            ApiError::Validation(
                "Código municipal para operação (cTribMun) é obrigatório quando o item informa CNA"
                    .into(),
            )
        })?;
    let municipal = normalize_municipal_code(municipal_raw)?;

    Ok(ItemTributacaoResolved {
        cna_id: Some(cna_id),
        codigo_tributacao_nacional: Some(nacional),
        codigo_tributacao_municipal: Some(municipal),
        conta_fiscal_id: None,
    })
}
