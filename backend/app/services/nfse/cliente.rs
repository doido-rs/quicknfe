use std::fs;
use std::time::Duration;

use reqwest::{Client, Identity};
use serde::Serialize;

use super::ambiente::AmbienteNfse;
use super::compactacao::xml_para_dps_gzip_b64;
use super::resposta::{interpretar_resposta_http, ResultadoTransmissao};
use crate::services::api_error::ApiError;

#[derive(Serialize)]
struct NfsePostRequest<'a> {
    #[serde(rename = "dpsXmlGZipB64")]
    dps_xml_gzip_b64: &'a str,
}

pub struct CredenciaisMtls {
    pub caminho_pfx: String,
    pub senha_pfx: String,
}

fn build_mtls_client(credenciais: &CredenciaisMtls) -> Result<Client, ApiError> {
    let pfx = fs::read(&credenciais.caminho_pfx).map_err(|e| {
        ApiError::Validation(format!(
            "não foi possível ler o certificado em {}: {e}",
            credenciais.caminho_pfx
        ))
    })?;
    let identity = Identity::from_pkcs12_der(&pfx, &credenciais.senha_pfx).map_err(|e| {
        ApiError::Validation(format!(
            "certificado PFX inválido ou senha incorreta ({}): {e}",
            credenciais.caminho_pfx
        ))
    })?;
    Client::builder()
        .identity(identity)
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|e| ApiError::Internal(format!("cliente HTTP mTLS: {e}")))
}

/// Envia DPS assinada (XML) para `POST /nfse` com mTLS.
pub async fn enviar_dps_assinada(
    xml_dps_assinado: &str,
    ambiente: AmbienteNfse,
    credenciais: &CredenciaisMtls,
) -> Result<ResultadoTransmissao, ApiError> {
    if !xml_dps_assinado.contains("<Signature") && !xml_dps_assinado.contains(":Signature") {
        return Err(ApiError::Validation(
            "xml_assinado deve conter assinatura XML-DSig (<Signature>) antes do envio".into(),
        ));
    }

    let dps_b64 = xml_para_dps_gzip_b64(xml_dps_assinado)?;
    let client = build_mtls_client(credenciais)?;
    let url = ambiente.url_emissao_nfse();
    let payload = NfsePostRequest {
        dps_xml_gzip_b64: &dps_b64,
    };

    let response = client
        .post(url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| ApiError::Internal(format!("falha na comunicação mTLS com a SEFIN: {e}")))?;

    let status = response.status().as_u16();
    let body = response
        .text()
        .await
        .map_err(|e| ApiError::Internal(format!("falha ao ler corpo da resposta SEFIN: {e}")))?;

    interpretar_resposta_http(status, &body)
}
