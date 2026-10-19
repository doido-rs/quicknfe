//! Validação e normalização de contas fiscais (cTribNac / cTribMun).

use doido::model::sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};

use crate::connectors::adn;
use crate::models::empresa::Entity as EmpresaEntity;
use crate::models::municipio_cna_parametrizacao::{Column as ParamColumn, Entity as ParamEntity};
use crate::services::api_error::ApiError;
use crate::services::cna::require_cna;
use crate::services::nfse::normalize_c_trib_nac;
use crate::services::nfse::municipio_tem_parametrizacao;

#[derive(Debug, Clone)]
pub struct ContaFiscalValidated {
    pub cna_id: i64,
    pub codigo_tributacao_nacional: String,
    pub codigo_tributacao_municipal: String,
}

pub fn normalize_municipal_code(raw: &str) -> Result<String, ApiError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(ApiError::Validation(
            "Código municipal para operação é obrigatório (cTribMun)".into(),
        ));
    }
    let mun = adn::normalize_c_trib_mun(trimmed);
    if mun == "000" {
        return Err(ApiError::Validation(
            "Código municipal para operação inválido: não use 000 (RN E0315)".into(),
        ));
    }
    Ok(mun)
}

pub async fn validar_conta_fiscal(
    db: &impl ConnectionTrait,
    empresa_id: i64,
    cna_id: i64,
    codigo_tributacao_nacional: Option<&str>,
    codigo_tributacao_municipal: &str,
) -> Result<ContaFiscalValidated, ApiError> {
    if cna_id <= 0 {
        return Err(ApiError::Validation(
            "Código CNA da operação (cna_id) é obrigatório".into(),
        ));
    }
    let cna = require_cna(db, cna_id).await?;
    let from_cna = normalize_c_trib_nac(&cna.codigo).ok_or_else(|| {
        ApiError::Validation(format!(
            "CNA {} possui código nacional inválido: {}",
            cna.id, cna.codigo
        ))
    })?;

    let nacional = match codigo_tributacao_nacional {
        None | Some("") => from_cna.clone(),
        Some(client) => {
            let normalized = normalize_c_trib_nac(client).ok_or_else(|| {
                ApiError::Validation(format!("Código de tributação nacional inválido: {client}"))
            })?;
            if normalized != from_cna {
                return Err(ApiError::Validation(format!(
                    "Código nacional {normalized} não corresponde ao CNA selecionado ({from_cna})"
                )));
            }
            normalized
        }
    };

    let municipal = normalize_municipal_code(codigo_tributacao_municipal)?;

    validar_parametrizacao_municipal(db, empresa_id, &nacional, &municipal).await?;

    Ok(ContaFiscalValidated {
        cna_id,
        codigo_tributacao_nacional: nacional,
        codigo_tributacao_municipal: municipal,
    })
}

async fn validar_parametrizacao_municipal(
    db: &impl ConnectionTrait,
    empresa_id: i64,
    codigo_trib_nac: &str,
    codigo_trib_mun: &str,
) -> Result<(), ApiError> {
    let prestador = EmpresaEntity::find_by_id(empresa_id)
        .one(db)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or(ApiError::NotFound)?;

    if !municipio_tem_parametrizacao(db, prestador.municipio_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
    {
        return Ok(());
    }

    let exists = ParamEntity::find()
        .filter(ParamColumn::MunicipioId.eq(prestador.municipio_id))
        .filter(ParamColumn::CodigoTribNac.eq(codigo_trib_nac))
        .filter(ParamColumn::CodigoTribMun.eq(codigo_trib_mun))
        .one(db)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .is_some();

    if !exists {
        return Err(ApiError::Validation(format!(
            "Combinação CNA {codigo_trib_nac} + código municipal {codigo_trib_mun} \
             não consta na parametrização do município do prestador (E0312/E0314)"
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_municipal_rejects_empty_and_000() {
        assert!(normalize_municipal_code("").is_err());
        assert!(normalize_municipal_code("000").is_err());
        assert_eq!(normalize_municipal_code("1").unwrap(), "001");
        assert_eq!(normalize_municipal_code("12").unwrap(), "012");
    }
}
