use std::collections::{HashMap, HashSet};

use doido::model::sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};

use crate::models::municipio::{Column as MunicipioColumn, Entity as MunicipioEntity, Model as Municipio};
use crate::services::api_error::ApiError;

pub async fn require_municipio(db: &impl ConnectionTrait, id: i64) -> Result<Municipio, ApiError> {
    MunicipioEntity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::Validation(format!("município {id} não encontrado")))
}

pub async fn map_by_ids(
    db: &impl ConnectionTrait,
    ids: impl IntoIterator<Item = i64>,
) -> Result<HashMap<i64, Municipio>, ApiError> {
    let unique: HashSet<i64> = ids.into_iter().collect();
    if unique.is_empty() {
        return Ok(HashMap::new());
    }
    let rows = MunicipioEntity::find()
        .filter(MunicipioColumn::Id.is_in(unique.iter().copied().collect::<Vec<_>>()))
        .all(db)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(rows.into_iter().map(|m| (m.id, m)).collect())
}
