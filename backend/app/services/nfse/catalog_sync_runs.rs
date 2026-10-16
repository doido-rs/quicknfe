//! Persistência de execuções de sync do catálogo NFS-e.

use chrono::Utc;
use doido::model::sea_orm::{ActiveModelTrait, ConnectionTrait, EntityTrait, Set};

use crate::models::nfse_catalog_sync_run::ActiveModel as RunActive;

pub async fn start_run(db: &impl ConnectionTrait, kind: &str) -> doido_core::Result<i64> {
    let row = RunActive {
        kind: Set(kind.to_string()),
        started_at: Set(Utc::now()),
        status: Set("running".into()),
        ..Default::default()
    }
    .insert(db)
    .await
    .map_err(|e| doido_core::anyhow::anyhow!("insert sync run: {e}"))?;
    Ok(row.id)
}

pub async fn finish_run(
    db: &impl ConnectionTrait,
    id: i64,
    status: &str,
    stats_json: Option<String>,
) -> doido_core::Result<()> {
    let row = crate::models::nfse_catalog_sync_run::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| doido_core::anyhow::anyhow!("find sync run: {e}"))?
        .ok_or_else(|| doido_core::anyhow::anyhow!("sync run {id} não encontrado"))?;
    let mut active: RunActive = row.into();
    active.finished_at = Set(Some(Utc::now()));
    active.status = Set(status.to_string());
    active.stats_json = Set(stats_json);
    active
        .update(db)
        .await
        .map_err(|e| doido_core::anyhow::anyhow!("update sync run: {e}"))?;
    Ok(())
}
