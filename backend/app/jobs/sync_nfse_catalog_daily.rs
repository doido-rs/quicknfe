//! Carga diária: municípios IBGE, lista nacional/incidência e fan-out municipal.

use chrono::Utc;
use doido_jobs::{job, JobContext};
use serde::{Deserialize, Serialize};

use crate::services::job_queue;
use crate::services::nfse::{catalog_daily, next_daily_catalog_run};
use crate::settings;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyncNfseCatalogDailyJobPayload {
    /// Quando true, não reagenda próxima execução (útil em testes manuais).
    #[serde(default)]
    pub skip_reschedule: bool,
}

#[job(queue = "catalog", max_retries = 1, timeout = 3600)]
pub async fn sync_nfse_catalog_daily(
    ctx: &JobContext,
    payload: SyncNfseCatalogDailyJobPayload,
) -> doido_core::Result<()> {
    let db = ctx.db();
    let stats = catalog_daily::run_nacional_import(db).await?;
    catalog_daily::record_nacional_run(db, stats.clone()).await?;

    let coordinator_payload = SyncMunicipioCnaCoordinatorJobPayload::default();
    let queue = job_queue::default_job_queue().await?;
    SyncMunicipioCnaCoordinatorJob::new(coordinator_payload)
        .enqueue(queue.as_ref())
        .await
        .map_err(|e| doido_core::anyhow::anyhow!("enqueue coordinator: {e}"))?;

    if !payload.skip_reschedule {
        let next = next_daily_catalog_run(Utc::now());
        SyncNfseCatalogDailyJob::new(SyncNfseCatalogDailyJobPayload::default())
            .at(next)
            .enqueue(queue.as_ref())
            .await
            .map_err(|e| doido_core::anyhow::anyhow!("reagendar sync diário: {e}"))?;
    }

    eprintln!("sync_nfse_catalog_daily concluído: {stats}");
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyncMunicipioCnaCoordinatorJobPayload {}

#[job(queue = "catalog", max_retries = 1, timeout = 600)]
pub async fn sync_municipio_cna_coordinator(
    ctx: &JobContext,
    _payload: SyncMunicipioCnaCoordinatorJobPayload,
) -> doido_core::Result<()> {
    let db = ctx.db();
    let cfg = settings::get();
    let codes = catalog_daily::list_all_ibge_codes(db).await?;
    let batches = catalog_daily::chunk_ibge(&codes, cfg.nfse_catalog.batch_size);
    let queue = job_queue::default_job_queue().await?;
    let mut enqueued = 0u32;
    for batch in batches {
        SyncMunicipioCnaLoteJob::new(SyncMunicipioCnaLoteJobPayload {
            ibge_codes: batch,
        })
        .enqueue(queue.as_ref())
        .await
        .map_err(|e| doido_core::anyhow::anyhow!("enqueue lote: {e}"))?;
        enqueued += 1;
    }
    catalog_daily::record_municipios_run(
        db,
        serde_json::json!({ "lotes_enfileirados": enqueued, "municipios": codes.len() }),
        "enqueued",
    )
    .await?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncMunicipioCnaLoteJobPayload {
    pub ibge_codes: Vec<String>,
}

#[job(queue = "catalog", max_retries = 2, timeout = 1800)]
pub async fn sync_municipio_cna_lote(
    ctx: &JobContext,
    payload: SyncMunicipioCnaLoteJobPayload,
) -> doido_core::Result<()> {
    let db = ctx.db();
    let sleep_ms = settings::get().nfse_catalog.sleep_ms;
    let results =
        crate::services::nfse::sync_municipios_batch(db, &payload.ibge_codes, sleep_ms).await;
    let ok = results.iter().filter(|r| r.error.is_none()).count();
    let err = results.len() - ok;
    if err > 0 {
        for r in results.iter().filter(|r| r.error.is_some()) {
            eprintln!(
                "lote municipal IBGE {}: {}",
                r.ibge,
                r.error.as_deref().unwrap_or("erro")
            );
        }
    }
    eprintln!(
        "lote municipal: {ok} ok, {err} erros ({} IBGE)",
        payload.ibge_codes.len()
    );
    Ok(())
}
