//! Carrega registros garantindo que pertencem à empresa do tenant.

use doido::model::sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use crate::models::certificado_digital::{Column as CertColumn, Entity as CertEntity};
use crate::models::cliente::{Column as ClienteColumn, Entity as ClienteEntity};
use crate::models::documento_fiscal::{Column as DocColumn, Entity as DocEntity};
use crate::models::evento_documento_fiscal::{Column as EventoColumn, Entity as EventoEntity};
use crate::models::intermediario::{Column as IntermediarioColumn, Entity as IntermediarioEntity};
use crate::models::item_documento_fiscal::{Column as ItemColumn, Entity as ItemEntity};
use crate::models::log_mensagem_retorno::Entity as LogEntity;
use crate::models::lote_transmissao::{Column as LoteColumn, Entity as LoteEntity};
use crate::models::tributo_item::{Column as TributoColumn, Entity as TributoEntity};

use super::api_error::ApiError;

pub async fn cliente_for_tenant(
    db: &DatabaseConnection,
    empresa_id: i64,
    id: i64,
) -> Result<crate::models::cliente::Model, ApiError> {
    ClienteEntity::find_by_id(id)
        .filter(ClienteColumn::EmpresaId.eq(empresa_id))
        .one(db)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or(ApiError::NotFound)
}

pub async fn intermediario_for_tenant(
    db: &DatabaseConnection,
    empresa_id: i64,
    id: i64,
) -> Result<crate::models::intermediario::Model, ApiError> {
    IntermediarioEntity::find_by_id(id)
        .filter(IntermediarioColumn::EmpresaId.eq(empresa_id))
        .one(db)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or(ApiError::NotFound)
}

pub async fn certificado_for_tenant(
    db: &DatabaseConnection,
    empresa_id: i64,
    id: i64,
) -> Result<crate::models::certificado_digital::Model, ApiError> {
    CertEntity::find_by_id(id)
        .filter(CertColumn::EmpresaId.eq(empresa_id))
        .one(db)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or(ApiError::NotFound)
}

pub async fn lote_for_tenant(
    db: &DatabaseConnection,
    empresa_id: i64,
    id: i64,
) -> Result<crate::models::lote_transmissao::Model, ApiError> {
    LoteEntity::find_by_id(id)
        .filter(LoteColumn::EmpresaId.eq(empresa_id))
        .one(db)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or(ApiError::NotFound)
}

pub async fn documento_for_tenant(
    db: &DatabaseConnection,
    empresa_id: i64,
    id: i64,
) -> Result<crate::models::documento_fiscal::Model, ApiError> {
    DocEntity::find_by_id(id)
        .filter(DocColumn::EmpresaId.eq(empresa_id))
        .one(db)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or(ApiError::NotFound)
}

pub async fn item_for_tenant(
    db: &DatabaseConnection,
    empresa_id: i64,
    item_id: i64,
) -> Result<crate::models::item_documento_fiscal::Model, ApiError> {
    let item = ItemEntity::find_by_id(item_id)
        .one(db)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or(ApiError::NotFound)?;
    documento_for_tenant(db, empresa_id, item.documento_fiscal_id).await?;
    Ok(item)
}

pub async fn item_for_tenant_and_doc(
    db: &DatabaseConnection,
    empresa_id: i64,
    doc_id: i64,
    item_id: i64,
) -> Result<crate::models::item_documento_fiscal::Model, ApiError> {
    documento_for_tenant(db, empresa_id, doc_id).await?;
    ItemEntity::find_by_id(item_id)
        .filter(ItemColumn::DocumentoFiscalId.eq(doc_id))
        .one(db)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or(ApiError::NotFound)
}

pub async fn tributo_for_tenant(
    db: &DatabaseConnection,
    empresa_id: i64,
    item_id: i64,
    tributo_id: i64,
) -> Result<crate::models::tributo_item::Model, ApiError> {
    item_for_tenant(db, empresa_id, item_id).await?;
    TributoEntity::find_by_id(tributo_id)
        .filter(TributoColumn::ItemId.eq(item_id))
        .one(db)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or(ApiError::NotFound)
}

pub async fn evento_for_tenant(
    db: &DatabaseConnection,
    empresa_id: i64,
    doc_id: i64,
    evento_id: i64,
) -> Result<crate::models::evento_documento_fiscal::Model, ApiError> {
    documento_for_tenant(db, empresa_id, doc_id).await?;
    EventoEntity::find_by_id(evento_id)
        .filter(EventoColumn::DocumentoFiscalId.eq(doc_id))
        .one(db)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or(ApiError::NotFound)
}

pub async fn log_for_tenant(
    db: &DatabaseConnection,
    empresa_id: i64,
    id: i64,
) -> Result<crate::models::log_mensagem_retorno::Model, ApiError> {
    let log = LogEntity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or(ApiError::NotFound)?;
    if let Some(doc_id) = log.documento_fiscal_id {
        documento_for_tenant(db, empresa_id, doc_id).await?;
    } else if let Some(lote_id) = log.lote_id {
        lote_for_tenant(db, empresa_id, lote_id).await?;
    } else {
        return Err(ApiError::NotFound);
    }
    Ok(log)
}

pub async fn ensure_cliente_belongs(
    db: &DatabaseConnection,
    empresa_id: i64,
    cliente_id: i64,
) -> Result<(), ApiError> {
    cliente_for_tenant(db, empresa_id, cliente_id).await.map(|_| ())
}

pub async fn ensure_lote_belongs(
    db: &DatabaseConnection,
    empresa_id: i64,
    lote_id: i64,
) -> Result<(), ApiError> {
    lote_for_tenant(db, empresa_id, lote_id).await.map(|_| ())
}

pub async fn ensure_intermediario_belongs(
    db: &DatabaseConnection,
    empresa_id: i64,
    intermediario_id: i64,
) -> Result<(), ApiError> {
    intermediario_for_tenant(db, empresa_id, intermediario_id)
        .await
        .map(|_| ())
}
