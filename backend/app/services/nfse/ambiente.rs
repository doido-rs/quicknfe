use crate::services::api_error::ApiError;
use crate::settings::{get as app_settings, normalize_base_url};

/// Ambiente da NFS-e Nacional (`tpAmb`: 1 = Produção, 2 = Homologação / Produção Restrita).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmbienteNfse {
    Producao,
    Homologacao,
}

impl AmbienteNfse {
    pub fn from_codigo(codigo: i32) -> Result<Self, ApiError> {
        match codigo {
            1 => Ok(Self::Producao),
            2 => Ok(Self::Homologacao),
            _ => Err(ApiError::Validation(format!(
                "ambiente inválido para NFS-e: {codigo} (use 1=produção ou 2=homologação)"
            ))),
        }
    }

    pub fn codigo_tp_amb(self) -> i32 {
        match self {
            Self::Producao => 1,
            Self::Homologacao => 2,
        }
    }

    /// URL base (sem barra final) — `settings.sefin` em `config/<env>.yml`.
    pub fn base_url(self) -> String {
        let sefin = &app_settings().sefin;
        let raw = match self {
            Self::Homologacao => &sefin.base_url_homologacao,
            Self::Producao => &sefin.base_url_producao,
        };
        normalize_base_url(raw)
    }

    pub fn url_emissao_nfse(self) -> String {
        format!("{}/nfse", self.base_url())
    }

    pub fn url_consulta_dps(self, id_dps: &str) -> String {
        format!("{}/dps/{}", self.base_url(), id_dps)
    }
}
