use std::time::Duration;

use reqwest::Client;

use crate::connectors::mtls::{build_mtls_client, CredenciaisMtls};
use crate::services::api_error::ApiError;

pub fn cnc_usa_mtls(base_url: &str) -> bool {
    let lower = base_url.to_ascii_lowercase();
    !(lower.contains("127.0.0.1") || lower.contains("localhost"))
}

fn http_client_plain() -> Result<Client, ApiError> {
    Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| ApiError::Internal(format!("cliente HTTP CNC: {e}")))
}

/// `GET` CNC cadastro — returns HTTP status and raw body text.
pub async fn get_cad(
    url: &str,
    base_url_for_mtls_policy: &str,
    credenciais: Option<CredenciaisMtls>,
) -> Result<(u16, String), ApiError> {
    let client = if cnc_usa_mtls(base_url_for_mtls_policy) {
        let cred = credenciais.ok_or_else(|| {
            ApiError::Validation(
                "certificado digital do prestador é obrigatório para consulta CNC".into(),
            )
        })?;
        build_mtls_client(&cred)?
    } else {
        http_client_plain()?
    };

    let response = client
        .get(url)
        .header("Accept", "application/json, text/plain")
        .send()
        .await
        .map_err(|e| ApiError::Internal(format!("falha na consulta CNC: {e}")))?;

    let status = response.status().as_u16();
    let body = response
        .text()
        .await
        .map_err(|e| ApiError::Internal(format!("falha ao ler resposta CNC: {e}")))?;

    Ok((status, body))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cnc_mock_nao_exige_mtls() {
        assert!(!cnc_usa_mtls("http://127.0.0.1:8090/cnc/consulta"));
        assert!(cnc_usa_mtls(
            "https://adn.producaorestrita.nfse.gov.br/cnc/consulta"
        ));
    }
}
