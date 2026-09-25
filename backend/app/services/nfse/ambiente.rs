use crate::services::api_error::ApiError;

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

    /// URL base oficial (sem barra final).
    pub fn base_url(self) -> &'static str {
        match self {
            Self::Homologacao => "https://sefin.producaorestrita.nfse.gov.br/API/SefinNacional",
            Self::Producao => "https://sefin.nfse.gov.br/SefinNacional",
        }
    }

    pub fn url_emissao_nfse(self) -> String {
        format!("{}/nfse", self.base_url())
    }
}
