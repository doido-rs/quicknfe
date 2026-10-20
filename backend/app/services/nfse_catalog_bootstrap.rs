//! Enfileira import Anexo A e sync diário do catálogo NFS-e (fila `catalog`).

use doido::model::sea_orm::{
    ConnectionTrait, DatabaseConnection, DbErr, FromQueryResult, QueryResult, Statement,
};

struct SchemaColCount {
    n: i64,
}

impl FromQueryResult for SchemaColCount {
    fn from_query_result(row: &QueryResult, pre: &str) -> Result<Self, DbErr> {
        Ok(Self {
            n: row.try_get(pre, "n")?,
        })
    }
}

async fn table_exists(db: &impl ConnectionTrait, table_name: &str) -> doido_core::Result<bool> {
    let backend = db.get_database_backend();
    let sql = format!(
        "SELECT COUNT(*) AS n FROM information_schema.TABLES \
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = '{table_name}'"
    );
    let row = SchemaColCount::find_by_statement(Statement::from_string(backend, sql))
        .one(db)
        .await
        .map_err(|e| doido_core::anyhow::anyhow!("{e}"))?;
    Ok(row.map(|r| r.n).unwrap_or(0) > 0)
}

async fn municipio_cna_parametrizacao_is_empty(db: &impl ConnectionTrait) -> doido_core::Result<bool> {
    if !table_exists(db, "municipio_cna_parametrizacao").await? {
        return Ok(true);
    }
    let backend = db.get_database_backend();
    let row = SchemaColCount::find_by_statement(Statement::from_string(
        backend,
        "SELECT COUNT(*) AS n FROM municipio_cna_parametrizacao".to_string(),
    ))
    .one(db)
    .await
    .map_err(|e| doido_core::anyhow::anyhow!("{e}"))?;
    Ok(row.map(|r| r.n).unwrap_or(0) == 0)
}

async fn pending_doido_job_count(
    db: &impl ConnectionTrait,
    queue: &str,
    job_name: &str,
) -> doido_core::Result<i64> {
    let backend = db.get_database_backend();
    let pattern = format!("%{job_name}%");
    let sql = format!(
        "SELECT COUNT(*) AS n FROM doido_jobs \
         WHERE queue = '{queue}' AND status IN ('pending','scheduled') \
         AND data LIKE '{pattern}'"
    );
    let pending = SchemaColCount::find_by_statement(Statement::from_string(backend, sql))
        .one(db)
        .await
        .map_err(|e| doido_core::anyhow::anyhow!("{e}"))?;
    Ok(pending.map(|r| r.n).unwrap_or(0))
}

/// Enfileira Anexo A (municípios/países) e `sync_nfse_catalog_daily` quando a parametrização municipal ainda está vazia.
pub async fn bootstrap_nfse_catalog_jobs(db: &DatabaseConnection) -> doido_core::Result<()> {
    if !municipio_cna_parametrizacao_is_empty(db).await? {
        return Ok(());
    }

    let queue = crate::services::job_queue::queue_for(db.clone()).await?;

    if pending_doido_job_count(db, "default", "import_nfse_anexo_a").await? == 0 {
        use crate::jobs::import_nfse_anexo_a::{
            ImportNfseAnexoAJob, ImportNfseAnexoAJobPayload,
        };
        ImportNfseAnexoAJob::new(ImportNfseAnexoAJobPayload::default())
            .enqueue(queue.as_ref())
            .await
            .map_err(|e| doido_core::anyhow::anyhow!("enqueue import anexo A: {e}"))?;
    }

    if pending_doido_job_count(db, "catalog", "sync_nfse_catalog_daily").await? == 0 {
        use crate::jobs::sync_nfse_catalog_daily::{
            SyncNfseCatalogDailyJob, SyncNfseCatalogDailyJobPayload,
        };
        SyncNfseCatalogDailyJob::new(SyncNfseCatalogDailyJobPayload::default())
            .enqueue(queue.as_ref())
            .await
            .map_err(|e| doido_core::anyhow::anyhow!("enqueue catalog sync: {e}"))?;
    }

    Ok(())
}
