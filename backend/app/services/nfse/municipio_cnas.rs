//! CNAs administrados por município (parametrização sincronizada).

use chrono::NaiveDate;
use doido::model::sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::models::cna::Entity as CnaEntity;
use crate::models::municipio_cna_parametrizacao::{Column as ParamColumn, Entity as ParamEntity};
use crate::services::api_error::ApiError;
use crate::services::dto::CnaDto;
use crate::services::nfse::e0312_validacao::vigente_em;

pub async fn list_cnas_administrados(
    db: &doido::model::sea_orm::DatabaseConnection,
    municipio_id: i64,
    competencia: Option<NaiveDate>,
) -> Result<Vec<CnaDto>, ApiError> {
    let params = ParamEntity::find()
        .filter(ParamColumn::MunicipioId.eq(municipio_id))
        .all(db)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    if params.is_empty() {
        return Ok(Vec::new());
    }

    let mut codigos: Vec<String> = params
        .iter()
        .filter(|p| {
            competencia
                .map(|c| vigente_em(c, p.vigencia_inicio, p.vigencia_fim))
                .unwrap_or(true)
        })
        .map(|p| p.codigo_trib_nac.clone())
        .collect();
    codigos.sort();
    codigos.dedup();

    if codigos.is_empty() {
        return Ok(Vec::new());
    }

    let cnas = CnaEntity::find()
        .filter(crate::models::cna::Column::Codigo.is_in(codigos))
        .all(db)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(cnas.into_iter().map(CnaDto::from).collect())
}
