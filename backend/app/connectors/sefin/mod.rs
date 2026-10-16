use serde::Serialize;

use crate::connectors::mtls::{build_mtls_client, CredenciaisMtls};
use crate::services::api_error::ApiError;

#[derive(Serialize)]
struct NfsePostRequest<'a> {
    #[serde(rename = "dpsXmlGZipB64")]
    dps_xml_gzip_b64: &'a str,
}

/// `POST` emissão NFS-e (DPS gzip+base64 JSON) com mTLS.
pub async fn post_nfse_emissao(
    url: &str,
    dps_xml_gzip_b64: &str,
    credenciais: &CredenciaisMtls,
) -> Result<(u16, String), ApiError> {
    let client = build_mtls_client(credenciais)?;
    let payload = NfsePostRequest {
        dps_xml_gzip_b64,
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

    Ok((status, body))
}

#[cfg(test)]
mod tests {
    use super::NfsePostRequest;

    #[test]
    fn nfse_post_request_json_field_name() {
        let req = NfsePostRequest {
            dps_xml_gzip_b64: "abc",
        };
        let v = serde_json::to_value(&req).expect("json");
        assert_eq!(
            v.get("dpsXmlGZipB64").and_then(|x| x.as_str()),
            Some("abc")
        );
    }
}
