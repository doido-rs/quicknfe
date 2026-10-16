use std::fs;
use std::time::Duration;

use reqwest::{Client, Identity};

use crate::services::api_error::ApiError;
use crate::services::openssl_legacy::ensure_legacy_providers_for_pkcs12;

#[derive(Debug, Clone)]
pub struct CredenciaisMtls {
    pub caminho_pfx: String,
    pub senha_pfx: String,
}

pub fn build_mtls_client(credenciais: &CredenciaisMtls) -> Result<Client, ApiError> {
    ensure_legacy_providers_for_pkcs12();
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
