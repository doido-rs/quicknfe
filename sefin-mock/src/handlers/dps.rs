use axum::{extract::{Path, State}, Json};

use crate::compression::utf8_to_gzip_b64;
use crate::error::ApiError;
use crate::models::DpsConsultaResponse;
use crate::AppState;

/// `GET .../dps/{id}` — consulta mockada da DPS processada.
pub async fn get_dps(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DpsConsultaResponse>, ApiError> {
    let record = state.store.get(&id).ok_or_else(|| ApiError::not_found(&id))?;

    let dps_xml_g_zip_b64 = utf8_to_gzip_b64(&record.dps_xml)?;
    let nfse_xml_g_zip_b64 = record
        .nfse_xml
        .as_ref()
        .map(|xml| utf8_to_gzip_b64(xml))
        .transpose()?;

    Ok(Json(DpsConsultaResponse {
        id_dps: record.id_dps,
        status: "PROCESSADO".into(),
        chave_acesso: record.chave_acesso,
        data_hora_processamento: record.processado_em,
        dps_xml_g_zip_b64,
        nfse_xml_g_zip_b64,
    }))
}
