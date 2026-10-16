use std::io::{Read, Write};

use base64::{engine::general_purpose::STANDARD, Engine as _};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;

use crate::services::api_error::ApiError;

/// Compacta o XML da DPS assinada no formato exigido pela API (`gzip` + Base64).
pub fn xml_para_dps_gzip_b64(xml: &str) -> Result<String, ApiError> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(xml.as_bytes())
        .map_err(|e| ApiError::Internal(format!("falha ao compactar DPS: {e}")))?;
    let compressed = encoder
        .finish()
        .map_err(|e| ApiError::Internal(format!("falha ao finalizar gzip da DPS: {e}")))?;
    Ok(STANDARD.encode(compressed))
}

/// Descompacta campos de resposta (`nfseXmlGZipB64`, etc.).
pub fn gzip_b64_para_xml(b64: &str) -> Result<String, ApiError> {
    let bytes = STANDARD
        .decode(b64.trim())
        .map_err(|e| ApiError::Internal(format!("Base64 inválido na resposta NFS-e: {e}")))?;
    let mut decoder = GzDecoder::new(bytes.as_slice());
    let mut xml = String::new();
    decoder
        .read_to_string(&mut xml)
        .map_err(|e| ApiError::Internal(format!("falha ao descompactar XML NFS-e: {e}")))?;
    Ok(xml)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_gzip_base64() {
        let original = r#"<?xml version="1.0"?><DPS xmlns="http://www.sped.fazenda.gov.br/nfse"><infDPS Id="x"/></DPS>"#;
        let b64 = xml_para_dps_gzip_b64(original).unwrap();
        let back = gzip_b64_para_xml(&b64).unwrap();
        assert_eq!(back, original);
    }
}
