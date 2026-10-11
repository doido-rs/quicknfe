use std::time::Duration;

use reqwest::Client;

use super::tipos::{ConsultaCadParams, RespostaConsultaCad};
use crate::services::api_error::ApiError;
use crate::services::nfse::{build_mtls_client, CredenciaisMtls};
use crate::settings;

fn apenas_digitos(s: &str) -> String {
    s.chars().filter(|c| c.is_ascii_digit()).collect()
}

fn cnc_usa_mtls(base_url: &str) -> bool {
    let lower = base_url.to_ascii_lowercase();
    !(lower.contains("127.0.0.1") || lower.contains("localhost"))
}

fn http_client_plain() -> Result<Client, ApiError> {
    Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| ApiError::Internal(format!("cliente HTTP CNC: {e}")))
}

pub async fn consulta_cad(
    params: ConsultaCadParams,
    credenciais: Option<CredenciaisMtls>,
) -> Result<RespostaConsultaCad, ApiError> {
    let inscricao = apenas_digitos(&params.inscricao_federal);
    if inscricao.len() != 11 && inscricao.len() != 14 {
        return Err(ApiError::Validation(
            "inscricaoFederal deve ser CPF (11 dígitos) ou CNPJ (14 dígitos)".into(),
        ));
    }

    let base = settings::get().cnc.base_url_normalized();
    let mut query = format!(
        "inscricaoFederal={}",
        urlencoding::encode(&inscricao)
    );
    if let Some(cod) = params.cod_municipio {
        query.push_str(&format!("&codMunicipio={cod}"));
    }
    if let Some(ref ind) = params.indicador_municipal {
        if !ind.trim().is_empty() {
            query.push_str(&format!(
                "&indicadorMunicipal={}",
                urlencoding::encode(ind.trim())
            ));
        }
    }
    let url = format!("{base}/cad?{query}");

    let client = if cnc_usa_mtls(&base) {
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

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| ApiError::Internal(format!("falha ao ler resposta CNC: {e}")))?;

    if !status.is_success() && status.as_u16() != 404 {
        return Err(ApiError::Validation(format!(
            "CNC retornou HTTP {}: {}",
            status.as_u16(),
            body.chars().take(500).collect::<String>()
        )));
    }

    serde_json::from_str::<RespostaConsultaCad>(&body).map_err(|e| {
        ApiError::Internal(format!(
            "resposta CNC não é JSON válido (HTTP {}): {e}; corpo: {}",
            status.as_u16(),
            body.chars().take(200).collect::<String>()
        ))
    })
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
