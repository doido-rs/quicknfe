//! Verificação de PKCS#12 (.pfx/.p12) e extração de validade.

use chrono::{DateTime, NaiveDateTime, Utc};
use openssl::pkcs12::Pkcs12;

use crate::services::api_error::ApiError;
use crate::services::openssl_legacy::ensure_legacy_providers_for_pkcs12;

/// Abre o PFX com a senha informada e retorna a validade (notAfter) do certificado.
pub fn verificar_pfx(bytes: &[u8], senha: &str) -> Result<DateTime<Utc>, ApiError> {
    if bytes.is_empty() {
        return Err(ApiError::Validation(
            "arquivo do certificado está vazio".into(),
        ));
    }
    ensure_legacy_providers_for_pkcs12();
    let pkcs12 = Pkcs12::from_der(bytes)
        .and_then(|p| p.parse2(senha))
        .map_err(|e| pfx_parse_error(e))?;
    if pkcs12.pkey.is_none() {
        return Err(ApiError::Validation("PFX sem chave privada".into()));
    }
    let cert = pkcs12
        .cert
        .ok_or_else(|| ApiError::Validation("PFX sem certificado".into()))?;
    data_validade_from_cert(&cert)
}

fn pfx_parse_error(e: openssl::error::ErrorStack) -> ApiError {
    let msg = e
        .errors()
        .iter()
        .map(|err| err.reason().unwrap_or("erro OpenSSL"))
        .collect::<Vec<_>>()
        .join("; ");
    let lower = msg.to_ascii_lowercase();
    if lower.contains("unsupported") || lower.contains("unknown cipher") || lower.contains("rc2") {
        return ApiError::Validation(
            "arquivo PFX usa criptografia legada (RC2/3DES); instale o módulo OpenSSL legacy \
             (ossl-modules) no servidor ou reexporte o certificado em formato AES"
                .into(),
        );
    }
    if lower.contains("mac verify failure") || lower.contains("invalid password") {
        return ApiError::Validation("senha do certificado incorreta".into());
    }
    ApiError::Validation(format!(
        "senha incorreta ou arquivo de certificado inválido ({msg})"
    ))
}

fn data_validade_from_cert(cert: &openssl::x509::X509) -> Result<DateTime<Utc>, ApiError> {
    let s = cert.not_after().to_string();
    let ndt = NaiveDateTime::parse_from_str(s.trim(), "%b %d %H:%M:%S %Y GMT").map_err(|e| {
        ApiError::Internal(format!("validade do certificado (notAfter): {e}"))
    })?;
    Ok(DateTime::from_naive_utc_and_offset(ndt, Utc))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn verificar_pfx_fixture_ok() {
        let pfx = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/test-sign.pfx");
        let bytes = fs::read(&pfx).expect("read pfx");
        let validade = verificar_pfx(&bytes, "test123").expect("verify");
        assert!(validade > Utc::now());
    }

    #[test]
    fn verificar_pfx_senha_errada() {
        let pfx = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/test-sign.pfx");
        let bytes = fs::read(&pfx).expect("read pfx");
        assert!(verificar_pfx(&bytes, "wrong").is_err());
    }

    /// PFX ICP-Brasil com RC2 (OpenSSL 3). Arquivo + senha via env (não commitar senha).
    #[test]
    fn verificar_pfx_legacy_rc2_env_fixture() {
        let Some(pwd) = std::env::var("QUICKNFE_LEGACY_PFX_PASSWORD").ok() else {
            return;
        };
        let pfx = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../certificado.pfx");
        if !pfx.exists() {
            return;
        }
        let bytes = fs::read(&pfx).expect("read pfx");
        let validade = verificar_pfx(&bytes, &pwd).expect("legacy rc2 pfx");
        assert!(validade > Utc::now());
    }
}
