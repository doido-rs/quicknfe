use super::ambiente::AmbienteNfse;
use super::compactacao::xml_para_dps_gzip_b64;
use super::resposta::{interpretar_resposta_http, ResultadoTransmissao};
use crate::connectors::sefin;
use crate::services::api_error::ApiError;

pub use crate::connectors::mtls::{build_mtls_client, CredenciaisMtls};

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
    let url = ambiente.url_emissao_nfse()?;
    let (status, body) = sefin::post_nfse_emissao(&url, &dps_b64, credenciais).await?;

    interpretar_resposta_http(status, &body)
}
