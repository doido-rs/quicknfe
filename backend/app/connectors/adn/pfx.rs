//! Carrega bytes do PFX A1 para mTLS ADN (env base64 ou arquivo em settings).

use std::fs;
use std::path::Path;

use base64::Engine;

use crate::services::api_error::ApiError;
use crate::settings;

const ENV_PFX_BASE64: &[&str] = &["ADN_MTLS_PFX_BASE64", "CERTIFICADO_PFX_BASE64"];
const ENV_PFX_PASSWORD: &[&str] = &["ADN_MTLS_PFX_PASSWORD", "CERTIFICADO_PFX_PASSWORD"];

pub fn adn_pfx_password() -> String {
    for key in ENV_PFX_PASSWORD {
        if let Ok(v) = std::env::var(key) {
            let t = v.trim();
            if !t.is_empty() {
                return t.to_string();
            }
        }
    }
    settings::get()
        .adn
        .mtls_pfx_password
        .as_deref()
        .unwrap_or("")
        .to_string()
}

/// PFX bruto: prioridade env base64, depois `settings.adn.mtls_pfx_path`.
pub fn load_adn_pfx_bytes() -> Result<Option<Vec<u8>>, ApiError> {
    if let Some(bytes) = load_pfx_from_env_base64()? {
        return Ok(Some(bytes));
    }
    let Some(path) = settings::get().adn.mtls_pfx_path.as_deref() else {
        return Ok(None);
    };
    let path = path.trim();
    if path.is_empty() {
        return Ok(None);
    }
    let bytes = fs::read(path).map_err(|e| {
        ApiError::Validation(format!(
            "não foi possível ler ADN_MTLS_PFX em {}: {e}",
            path
        ))
    })?;
    Ok(Some(bytes))
}

fn load_pfx_from_env_base64() -> Result<Option<Vec<u8>>, ApiError> {
    for key in ENV_PFX_BASE64 {
        let Ok(raw) = std::env::var(key) else {
            continue;
        };
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(trimmed)
            .map_err(|e| ApiError::Validation(format!("{key} base64 inválido: {e}")))?;
        if bytes.is_empty() {
            return Err(ApiError::Validation(format!("{key} está vazio após decode")));
        }
        return Ok(Some(bytes));
    }
    Ok(None)
}

pub fn adn_pfx_configured() -> bool {
    load_pfx_from_env_base64()
        .ok()
        .flatten()
        .is_some()
        || settings::get()
            .adn
            .mtls_pfx_path
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .and_then(|p| Path::new(p).exists().then_some(()))
            .is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_roundtrip_sample() {
        let sample = b"PK-test-pfx-bytes";
        let b64 = base64::engine::general_purpose::STANDARD.encode(sample);
        std::env::set_var("ADN_MTLS_PFX_BASE64", &b64);
        let out = load_pfx_from_env_base64().expect("ok").expect("some");
        assert_eq!(out, sample);
        std::env::remove_var("ADN_MTLS_PFX_BASE64");
    }
}
