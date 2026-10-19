//! Configuração da aplicação no bloco `settings:` de `config/<env>.yml`.

use std::sync::OnceLock;

use doido_core::Environment;
use serde::Deserialize;

static SETTINGS: OnceLock<Settings> = OnceLock::new();

/// Raiz do bloco `settings:` nos YAML de ambiente.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub sefin: SefinSettings,
    pub cnc: CncSettings,
    pub adn: AdnSettings,
    pub nfse_catalog: NfseCatalogSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            sefin: SefinSettings::default(),
            cnc: CncSettings::default(),
            adn: AdnSettings::default(),
            nfse_catalog: NfseCatalogSettings::default(),
        }
    }
}

/// API ADN — consulta CNC (`GET .../cad`).
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct CncSettings {
    /// URL base até `/cnc/consulta` (sem barra final).
    pub base_url: String,
}

impl Default for CncSettings {
    fn default() -> Self {
        Self {
            base_url: "https://adn.producaorestrita.nfse.gov.br/cnc/consulta".into(),
        }
    }
}

impl CncSettings {
    pub fn base_url_normalized(&self) -> String {
        normalize_base_url(&self.base_url)
    }
}

/// Endpoint único da API SEFIN Nacional para esta instância (`tpAmb` 1 ou 2 + URL base).
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct SefinSettings {
    /// `1` = produção, `2` = homologação / produção restrita.
    pub ambiente: i32,
    /// URL base (sem barra final) usada em todas as chamadas SEFIN deste deploy.
    pub base_url: String,
}

impl Default for SefinSettings {
    fn default() -> Self {
        Self {
            ambiente: 2,
            base_url: "https://sefin.producaorestrita.nfse.gov.br/API/SefinNacional".into(),
        }
    }
}

impl SefinSettings {
    pub fn base_url_normalized(&self) -> String {
        normalize_base_url(&self.base_url)
    }
}

/// API ADN — parametrização municipal (alíquotas / códigos administrados).
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AdnSettings {
    pub base_url: String,
    /// Caminho do PFX A1 para mTLS nas chamadas ADN (produção/homolog). Vazio = sem mTLS.
    pub mtls_pfx_path: Option<String>,
    /// Senha do PFX (`ADN_MTLS_PFX_PASSWORD`); ignorada se `mtls_pfx_path` vazio.
    pub mtls_pfx_password: Option<String>,
}

impl Default for AdnSettings {
    fn default() -> Self {
        Self {
            base_url: "https://adn.producaorestrita.nfse.gov.br".into(),
            mtls_pfx_path: None,
            mtls_pfx_password: None,
        }
    }
}

impl AdnSettings {
    pub fn base_url_normalized(&self) -> String {
        normalize_base_url(&self.base_url)
    }

    /// Preferir [`crate::connectors::adn::pfx::load_adn_pfx_bytes`] (env base64 ou path).
    pub fn mtls_credentials(&self) -> Option<crate::connectors::mtls::CredenciaisMtls> {
        let path = self.mtls_pfx_path.as_deref()?.trim();
        if path.is_empty() {
            return None;
        }
        let senha = self.mtls_pfx_password.as_deref().unwrap_or("").to_string();
        Some(crate::connectors::mtls::CredenciaisMtls {
            caminho_pfx: path.to_string(),
            senha_pfx: senha,
        })
    }
}

/// Sync diário: lista nacional, incidência e parametrização por município.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct NfseCatalogSettings {
    /// Fila doido-jobs para jobs de catálogo.
    pub queue: String,
    /// Tamanho do lote de municípios por job filho.
    pub batch_size: u32,
    /// Pausa entre chamadas ADN por município (ms).
    pub sleep_ms: u64,
    /// Hora local (America/Sao_Paulo) para próxima execução diária (0–23).
    pub daily_hour: u32,
    /// URL do Anexo A (municípios IBGE); vazio = padrão em `nfse_anexo_a_import`.
    pub anexo_a_url: Option<String>,
    /// URL planilha com aba MUN.INCID_INFO.SERV.; vazio = `layout_dps.xlsx` no repo.
    pub incidencia_xlsx_url: Option<String>,
    /// URL Anexo B (lista nacional); opcional — incidência cobre códigos se falhar.
    pub anexo_b_url: Option<String>,
    /// Quando `…000` não retorna alíquota (ex. RJ), varre desdobros `001`… até este limite.
    pub desdobro_scan_max: u32,
    /// Interrompe a varredura após N desdobros consecutivos sem parametrização ADN.
    pub desdobro_scan_stop_after_misses: u32,
}

impl Default for NfseCatalogSettings {
    fn default() -> Self {
        Self {
            queue: "catalog".into(),
            batch_size: 50,
            sleep_ms: 100,
            daily_hour: 3,
            anexo_a_url: None,
            incidencia_xlsx_url: None,
            anexo_b_url: None,
            desdobro_scan_max: 100,
            desdobro_scan_stop_after_misses: 12,
        }
    }
}

pub fn normalize_base_url(url: &str) -> String {
    url.trim().trim_end_matches('/').to_string()
}

#[derive(Deserialize)]
struct ConfigRoot {
    #[serde(default)]
    settings: Settings,
}

fn load_from_config_file() -> Settings {
    let env = Environment::get_env();
    let path = format!("config/{}.yml", env.as_str());
    let raw = match std::fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(e) => {
            eprintln!("warning: config {path} não encontrado ({e}); usando settings padrão");
            return Settings::default();
        }
    };
    let rendered = match doido_core::config::render(&raw) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("warning: falha ao renderizar {path}: {e}; usando settings padrão");
            return Settings::default();
        }
    };
    match serde_norway::from_str::<ConfigRoot>(&rendered) {
        Ok(root) => normalize_settings(root.settings),
        Err(e) => {
            eprintln!("warning: falha ao ler settings de {path}: {e}; usando settings padrão");
            Settings::default()
        }
    }
}

fn blank_to_none(value: Option<String>) -> Option<String> {
    value.filter(|s| !s.trim().is_empty())
}

fn normalize_settings(mut settings: Settings) -> Settings {
    settings.adn.mtls_pfx_path = blank_to_none(settings.adn.mtls_pfx_path);
    settings.adn.mtls_pfx_password = blank_to_none(settings.adn.mtls_pfx_password);
    settings.nfse_catalog.anexo_a_url = blank_to_none(settings.nfse_catalog.anexo_a_url);
    settings.nfse_catalog.incidencia_xlsx_url =
        blank_to_none(settings.nfse_catalog.incidencia_xlsx_url);
    settings.nfse_catalog.anexo_b_url = blank_to_none(settings.nfse_catalog.anexo_b_url);
    settings
}

/// Carrega (ou reutiliza) settings do YAML do ambiente atual.
pub fn init() -> &'static Settings {
    SETTINGS.get_or_init(load_from_config_file)
}

pub fn get() -> &'static Settings {
    init()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_trims_slashes() {
        assert_eq!(
            normalize_base_url("  http://127.0.0.1:8090/API/SefinNacional/  "),
            "http://127.0.0.1:8090/API/SefinNacional"
        );
    }

    #[test]
    fn blank_catalog_urls_become_none() {
        let mut s = Settings::default();
        s.nfse_catalog.anexo_a_url = Some("   ".into());
        s.nfse_catalog.incidencia_xlsx_url = Some("".into());
        let s = normalize_settings(s);
        assert!(s.nfse_catalog.anexo_a_url.is_none());
        assert!(s.nfse_catalog.incidencia_xlsx_url.is_none());
    }
}
