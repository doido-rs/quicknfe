use axum::{extract::State, http::StatusCode, Json};
use chrono::Utc;
use uuid::Uuid;

use crate::compression::{gzip_b64_to_utf8, utf8_to_gzip_b64};
use crate::dps_id::{extrair_id_dps, gerar_chave_acesso_mock, montar_nfse_xml};
use crate::error::ApiError;
use crate::models::{MensagemProcessamento, NfsePostRequest, NfsePostResponse, StoredDps};
use crate::AppState;

/// `POST .../nfse` — emissão mockada (compatível com payload `dpsXmlGZipB64`).
pub async fn post_nfse(
    State(state): State<AppState>,
    Json(body): Json<NfsePostRequest>,
) -> Result<(StatusCode, Json<NfsePostResponse>), ApiError> {
    if body.dps_xml_g_zip_b64.trim().is_empty() {
        return Err(ApiError::bad_request("campo dpsXmlGZipB64 é obrigatório"));
    }

    let dps_xml = gzip_b64_to_utf8(&body.dps_xml_g_zip_b64)?;

    if !dps_xml.contains("<Signature") && !dps_xml.contains(":Signature") {
        return Err(ApiError::bad_request(
            "DPS deve conter assinatura XML-DSig (<Signature>)",
        ));
    }

    let id_dps = extrair_id_dps(&dps_xml).unwrap_or_else(|| format!("DPS{}", Uuid::new_v4().simple()));
    let chave_acesso = gerar_chave_acesso_mock();
    let nfse_xml = montar_nfse_xml(&chave_acesso, &id_dps);
    let nfse_xml_g_zip_b64 = utf8_to_gzip_b64(&nfse_xml)?;
    let now = Utc::now();

    state.store.insert(StoredDps {
        id_dps: id_dps.clone(),
        chave_acesso: chave_acesso.clone(),
        dps_xml: dps_xml.clone(),
        nfse_xml: Some(nfse_xml),
        processado_em: now,
    });

    let alertas = vec![MensagemProcessamento {
        codigo: Some("MOCK001".into()),
        descricao: Some("Resposta gerada pelo sefin-mock".into()),
        mensagem: Some("Ambiente de desenvolvimento local".into()),
        complemento: None,
    }];

    Ok((
        StatusCode::CREATED,
        Json(NfsePostResponse {
            tipo_ambiente: 2,
            versao_aplicativo: "sefin-mock/0.1".into(),
            data_hora_processamento: now,
            id_dps,
            chave_acesso,
            nfse_xml_g_zip_b64,
            alertas: Some(alertas),
        }),
    ))
}
