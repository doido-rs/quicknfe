//! OpenSSL 3+ não decodifica PKCS#12 antigos (ex. RC2-40-CBC) sem o provider *legacy*.

use std::sync::OnceLock;

static LEGACY_PROVIDER: OnceLock<Option<openssl::provider::Provider>> = OnceLock::new();

fn provider_modules_search_path() -> Option<String> {
    std::env::var("OPENSSL_MODULES").ok().or_else(|| {
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        {
            Some("/usr/lib/x86_64-linux-gnu/ossl-modules".into())
        }
        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        {
            Some("/usr/lib/aarch64-linux-gnu/ossl-modules".into())
        }
        #[cfg(not(any(
            all(target_os = "linux", target_arch = "x86_64"),
            all(target_os = "linux", target_arch = "aarch64"),
        )))]
        {
            None
        }
    })
}

fn load_legacy_provider() -> Result<openssl::provider::Provider, openssl::error::ErrorStack> {
    if let Some(path) = provider_modules_search_path() {
        let _ = openssl::provider::Provider::set_default_search_path(None, &path);
    }
    openssl::provider::Provider::try_load(None, "legacy", true)
}

/// Carrega e mantém o provider OpenSSL `legacy` (necessário para PFX ICP-Brasil antigos).
pub fn ensure_legacy_providers_for_pkcs12() {
    LEGACY_PROVIDER.get_or_init(|| load_legacy_provider().ok());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_provider_loads_when_available() {
        ensure_legacy_providers_for_pkcs12();
        assert!(
            LEGACY_PROVIDER.get().and_then(|p| p.as_ref()).is_some(),
            "legacy provider should load on OpenSSL 3 with ossl-modules installed"
        );
        let bytes = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../certificado.pfx"),
        )
        .ok();
        let Some(bytes) = bytes else { return };
        let pwd = std::env::var("QUICKNFE_LEGACY_PFX_PASSWORD").unwrap_or_default();
        if pwd.is_empty() {
            return;
        }
        let pkcs12 = openssl::pkcs12::Pkcs12::from_der(&bytes).expect("der");
        pkcs12.parse2(&pwd).expect("legacy provider must decode RC2 PFX");
    }
}
