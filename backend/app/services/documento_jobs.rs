//! Jobs NFSe correlacionados ao documento (estado in-flight + fila doido quando aplicável).

use chrono::Utc;
use serde::Serialize;

use crate::models::documento_fiscal::Model as Documento;
use crate::services::nfse::documento_status;

#[derive(Debug, Serialize)]
pub struct DocumentoJobRow {
    pub id: String,
    pub queue: String,
    pub status: String,
    pub run_at: chrono::DateTime<Utc>,
    pub job_class: Option<String>,
}

/// Retorna jobs ativos inferidos do `status` do documento (assinando / transmitindo).
pub fn list_jobs_for_documento(doc: &Documento) -> Vec<DocumentoJobRow> {
    let now = Utc::now();
    match doc.status.as_str() {
        documento_status::ASSINANDO => vec![DocumentoJobRow {
            id: format!("documento-{}-assinar", doc.id),
            queue: "default".into(),
            status: "pending".into(),
            run_at: now,
            job_class: Some("assinar_documento".into()),
        }],
        documento_status::TRANSMITINDO => vec![DocumentoJobRow {
            id: format!("documento-{}-transmitir", doc.id),
            queue: "default".into(),
            status: "pending".into(),
            run_at: now,
            job_class: Some("transmitir_documento_nfse".into()),
        }],
        _ => Vec::new(),
    }
}
