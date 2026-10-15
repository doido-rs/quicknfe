//! Enfileira assinatura/transmissão NFSe (API → doido_jobs).

use doido::model::sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

use crate::jobs::{assinar_documento, transmitir_documento};
use crate::models::documento_fiscal::{
    ActiveModel as DocActive, Column as DocColumn, Entity as DocEntity,
};
use crate::services::api_error::ApiError;
use crate::services::job_queue;
use crate::services::nfse::{
    documento_status, reject_in_flight, DocumentoAssinarParams, DocumentoEnqueueResponse,
};
use crate::services::tenant_scope;

pub async fn enqueue_assinar(
    db: &doido::model::sea_orm::DatabaseConnection,
    params: DocumentoAssinarParams,
) -> Result<DocumentoEnqueueResponse, ApiError> {
    let doc =
        tenant_scope::documento_for_company(db, params.company_id, params.documento_id).await?;
    reject_in_flight(&doc.status)?;
    if doc.status == documento_status::AUTORIZADA {
        return Err(ApiError::Validation(
            "Nota já autorizada pela SEFIN.".into(),
        ));
    }

    let mut record: DocActive = doc.into();
    record.status = Set(documento_status::ASSINANDO.into());
    record.processamento_mensagem = Set(Some("Na fila para assinatura".into()));
    let updated = record
        .update(db)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let payload = assinar_documento::AssinarDocumentoJobPayload {
        documento_id: params.documento_id,
        company_id: params.company_id,
        certificado_id: params.certificado_id,
    };
    let job_id = assinar_documento::AssinarDocumentoJob::new(payload)
        .enqueue(job_queue::default_job_queue().as_ref())
        .await
        .map_err(|e| ApiError::Internal(format!("falha ao enfileirar assinatura: {e}")))?;

    Ok(DocumentoEnqueueResponse {
        documento_id: updated.id,
        status: updated.status,
        job_id: Some(job_id),
        processamento_mensagem: updated.processamento_mensagem,
    })
}

pub async fn enqueue_transmitir(
    db: &doido::model::sea_orm::DatabaseConnection,
    company_id: i64,
    documento_id: i64,
    certificado_id: Option<i64>,
) -> Result<DocumentoEnqueueResponse, ApiError> {
    let doc = tenant_scope::documento_for_company(db, company_id, documento_id).await?;
    reject_in_flight(&doc.status)?;
    if doc.status == documento_status::AUTORIZADA {
        return Err(ApiError::Validation(
            "Nota já autorizada pela SEFIN.".into(),
        ));
    }
    if doc
        .xml_assinado
        .as_ref()
        .is_none_or(|x| !x.contains("Signature"))
    {
        return Err(ApiError::Validation(
            "Assine a nota fiscal antes de transmitir à SEFIN.".into(),
        ));
    }

    let mut record: DocActive = doc.into();
    record.status = Set(documento_status::TRANSMITINDO.into());
    record.processamento_mensagem = Set(Some("Na fila para transmissão SEFIN".into()));
    let updated = record
        .update(db)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let payload = transmitir_documento::TransmitirDocumentoNfseJobPayload {
        documento_id,
        company_id,
        certificado_id,
    };
    let job_id = transmitir_documento::TransmitirDocumentoNfseJob::new(payload)
        .enqueue(job_queue::default_job_queue().as_ref())
        .await
        .map_err(|e| ApiError::Internal(format!("falha ao enfileirar transmissão: {e}")))?;

    Ok(DocumentoEnqueueResponse {
        documento_id: updated.id,
        status: updated.status,
        job_id: Some(job_id),
        processamento_mensagem: updated.processamento_mensagem,
    })
}

pub async fn enqueue_transmitir_lote(
    db: &doido::model::sea_orm::DatabaseConnection,
    company_id: i64,
    prestador_id: i64,
    lote_id: i64,
) -> Result<Vec<DocumentoEnqueueResponse>, ApiError> {
    tenant_scope::lote_for_tenant(db, prestador_id, lote_id).await?;
    let docs = DocEntity::find()
        .filter(DocColumn::LoteId.eq(lote_id))
        .filter(DocColumn::EmpresaId.eq(prestador_id))
        .all(db)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    if docs.is_empty() {
        return Err(ApiError::Validation(
            "Lote não possui notas fiscais vinculadas.".into(),
        ));
    }
    let mut out = Vec::with_capacity(docs.len());
    for doc in docs {
        match enqueue_transmitir(db, company_id, doc.id, None).await {
            Ok(r) => out.push(r),
            Err(e) => out.push(DocumentoEnqueueResponse {
                documento_id: doc.id,
                status: doc.status,
                job_id: None,
                processamento_mensagem: Some(e.to_string()),
            }),
        }
    }
    Ok(out)
}
