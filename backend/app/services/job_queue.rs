//! Fila de jobs (`doido_jobs`) para enqueue a partir da API.

use std::sync::{Arc, OnceLock};

use doido_jobs::config::build_db_queue;
use doido_jobs::JobQueue;

static DEFAULT_QUEUE: OnceLock<Arc<dyn JobQueue>> = OnceLock::new();

pub fn default_job_queue() -> Arc<dyn JobQueue> {
    DEFAULT_QUEUE
        .get_or_init(|| build_db_queue(doido::model::pool::pool().clone()))
        .clone()
}
