//! Importa municípios (TAB.MUN_IBGE) e países (TAB.PAÍS_ISO2) do Anexo A oficial NFS-e.
//!
//! Enfileirar (com worker `cargo doido worker` rodando):
//! `ImportNfseAnexoAJob::new(ImportNfseAnexoAJobPayload::default()).enqueue(queue).await`
//! ou payload `{ "url": "https://..." }` para outra planilha.

use doido_jobs::{job, JobContext};
use serde::{Deserialize, Serialize};

use crate::services::nfse_anexo_a_import;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImportNfseAnexoAJobPayload {
    /// URL do `.xlsx`; padrão = documentação oficial no gov.br.
    #[serde(default)]
    pub url: Option<String>,
}

#[job(queue = "default", max_retries = 2, timeout = 900)]
pub async fn import_nfse_anexo_a(
    ctx: &JobContext,
    payload: ImportNfseAnexoAJobPayload,
) -> doido_core::Result<()> {
    let _stats = nfse_anexo_a_import::run(ctx.db(), payload.url.as_deref()).await?;
    Ok(())
}
