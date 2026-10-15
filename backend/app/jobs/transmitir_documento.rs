use doido_jobs::{job, JobContext};
use serde::{Deserialize, Serialize};

use crate::services::nfse::{marcar_erro_transmissao, transmitir_documento, TransmitirParams};
use crate::services::tenant_scope;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransmitirDocumentoNfseJobPayload {
    pub documento_id: i64,
    pub company_id: i64,
    pub certificado_id: Option<i64>,
}

#[job(queue = "default", max_retries = 2, timeout = 180)]
pub async fn transmitir_documento_nfse(
    ctx: &JobContext,
    payload: TransmitirDocumentoNfseJobPayload,
) -> doido_core::Result<()> {
    let db = ctx.db();
    let doc = tenant_scope::documento_for_company(db, payload.company_id, payload.documento_id)
        .await
        .map_err(|e| doido_core::anyhow::anyhow!("{e}"))?;
    match transmitir_documento(
        db,
        TransmitirParams {
            empresa_id: doc.empresa_id,
            documento_id: payload.documento_id,
            certificado_id: payload.certificado_id,
        },
    )
    .await
    {
        Ok(outcome) => {
            if !outcome.resultado.sucesso {
                let msg = outcome
                    .resultado
                    .erros
                    .first()
                    .map(|m| m.resumo())
                    .unwrap_or_else(|| "Transmissão SEFIN rejeitada".into());
                let _ = marcar_erro_transmissao(db, payload.documento_id, &msg).await;
                return Err(doido_core::anyhow::anyhow!("{msg}"));
            }
            Ok(())
        }
        Err(e) => {
            let msg = e.to_string();
            let _ = marcar_erro_transmissao(db, payload.documento_id, &msg).await;
            Err(doido_core::anyhow::anyhow!("{msg}"))
        }
    }
}
