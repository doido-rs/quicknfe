use doido_jobs::{job, JobContext};
use serde::{Deserialize, Serialize};

use crate::services::nfse::{assinar_documento_nfse, marcar_erro_assinatura};
use crate::services::tenant_scope;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssinarDocumentoJobPayload {
    pub documento_id: i64,
    pub company_id: i64,
    pub certificado_id: Option<i64>,
}

#[job(queue = "default", max_retries = 2, timeout = 120)]
pub async fn assinar_documento(
    ctx: &JobContext,
    payload: AssinarDocumentoJobPayload,
) -> doido_core::Result<()> {
    let db = ctx.db();
    let doc = tenant_scope::documento_for_company(db, payload.company_id, payload.documento_id)
        .await
        .map_err(|e| doido_core::anyhow::anyhow!("{e}"))?;
    match assinar_documento_nfse(
        db,
        doc,
        payload.certificado_id,
    )
    .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            let msg = e.to_string();
            let _ = marcar_erro_assinatura(db, payload.documento_id, &msg).await;
            Err(doido_core::anyhow::anyhow!("{msg}"))
        }
    }
}
