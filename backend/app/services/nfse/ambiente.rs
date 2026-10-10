use crate::services::api_error::ApiError;
use crate::settings::get as app_settings;

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

    /// Ambiente SEFIN configurado em `settings.sefin.ambiente`.
    pub fn configurado() -> Result<Self, ApiError> {
        Self::from_codigo(app_settings().sefin.ambiente)
    }

    /// Garante que o ambiente do documento coincide com o endpoint configurado.
    pub fn ensure_matches_config(self) -> Result<(), ApiError> {
        let configured = Self::configurado()?;
        if self != configured {
            return Err(ApiError::Validation(format!(
                "documento com ambiente {} (tpAmb), mas a API SEFIN está configurada para ambiente {} — ajuste settings.sefin ou o documento",
                self.codigo_tp_amb(),
                configured.codigo_tp_amb()
            )));
        }
        Ok(())
    }

    /// URL base (sem barra final) — único endpoint em `settings.sefin.base_url`.
    pub fn base_url(self) -> Result<String, ApiError> {
        self.ensure_matches_config()?;
        Ok(app_settings().sefin.base_url_normalized())
    }

    pub fn url_emissao_nfse(self) -> Result<String, ApiError> {
        Ok(format!("{}/nfse", self.base_url()?))
    }

    pub fn url_consulta_dps(self, id_dps: &str) -> Result<String, ApiError> {
        Ok(format!("{}/dps/{}", self.base_url()?, id_dps))
    }
}
