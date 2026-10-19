//! Fila de jobs (`doido_jobs`) para enqueue a partir da API.
//!
//! A tabela é criada/atualizada por `doido_jobs::db::DbQueue::migrate` (mesmo caminho do worker).

use std::sync::Arc;

use doido_jobs::db::DbQueue;
use doido_jobs::JobQueue;
use tokio::sync::OnceCell;

static DEFAULT_QUEUE: OnceCell<Arc<dyn JobQueue>> = OnceCell::const_new();

async fn migrate_db_queue(queue: &DbQueue) -> doido::Result<()> {
    queue
        .migrate()
        .await
        .map_err(|e| doido::model::sea_orm::DbErr::Custom(format!("doido_jobs migrate: {e}")))?;
    Ok(())
}

/// Fila global (pool HTTP/worker); garante `doido_jobs` na primeira chamada.
pub async fn default_job_queue() -> doido::Result<Arc<dyn JobQueue>> {
    Ok(DEFAULT_QUEUE
        .get_or_init(|| async {
            let conn = doido::model::pool::pool().clone();
            let queue = DbQueue::new(conn);
            migrate_db_queue(&queue)
                .await
                .expect("doido_jobs migrate failed");
            Arc::new(queue) as Arc<dyn JobQueue>
        })
        .await
        .clone())
}

/// Fila para enqueue fora do HTTP/worker (ex.: `db seed`), quando o pool global ainda não foi init.
pub async fn queue_for(db: doido::model::sea_orm::DatabaseConnection) -> doido::Result<Arc<dyn JobQueue>> {
    let queue = DbQueue::new(db);
    migrate_db_queue(&queue).await?;
    Ok(Arc::new(queue))
}
