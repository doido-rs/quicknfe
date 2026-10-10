use std::io::{Read, Write};

use base64::{engine::general_purpose::STANDARD, Engine as _};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;

use crate::error::ApiError;

pub fn gzip_b64_to_utf8(b64: &str) -> Result<String, ApiError> {
    let bytes = STANDARD
        .decode(b64.trim())
        .map_err(|e| ApiError::bad_request(format!("dpsXmlGZipB64 inválido (base64): {e}")))?;
    let mut decoder = GzDecoder::new(bytes.as_slice());
    let mut xml = String::new();
    decoder
        .read_to_string(&mut xml)
        .map_err(|e| ApiError::bad_request(format!("dpsXmlGZipB64 inválido (gzip): {e}")))?;
    Ok(xml)
}

pub fn utf8_to_gzip_b64(text: &str) -> Result<String, ApiError> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(text.as_bytes())
        .map_err(|e| ApiError::internal(format!("falha ao compactar XML: {e}")))?;
    let compressed = encoder
        .finish()
        .map_err(|e| ApiError::internal(format!("falha ao finalizar gzip: {e}")))?;
    Ok(STANDARD.encode(compressed))
}
