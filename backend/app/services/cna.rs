use doido::model::sea_orm::{ConnectionTrait, EntityTrait};

use crate::models::cna::Entity as CnaEntity;
use crate::services::api_error::ApiError;

pub async fn require_cna(
    db: &impl ConnectionTrait,
    id: i64,
) -> Result<crate::models::cna::Model, ApiError> {
    CnaEntity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or(ApiError::NotFound)
}
