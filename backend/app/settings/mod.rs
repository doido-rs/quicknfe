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
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            sefin: SefinSettings::default(),
        }
    }
}

/// URLs base da API SEFIN Nacional (sem barra final).
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct SefinSettings {
    pub base_url_homologacao: String,
    pub base_url_producao: String,
}

impl Default for SefinSettings {
    fn default() -> Self {
        Self {
            base_url_homologacao:
                "https://sefin.producaorestrita.nfse.gov.br/API/SefinNacional".into(),
            base_url_producao: "https://sefin.nfse.gov.br/SefinNacional".into(),
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
        Ok(root) => root.settings,
        Err(e) => {
            eprintln!("warning: falha ao ler settings de {path}: {e}; usando settings padrão");
            Settings::default()
        }
    }
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
}
