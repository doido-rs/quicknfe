//! Erros estruturados das regras RN (código SEFIN + campos de formulário).

use crate::services::api_error::{ApiError, ValidationRnError};
use crate::services::nfse::dps_rn_catalog::{rule_by_cod, RnRuleMeta};

#[derive(Debug, Clone)]
pub struct RnViolation {
    pub cod_erro: String,
    pub message: String,
    pub fields: Vec<String>,
}

impl RnViolation {
    pub fn from_meta(meta: &RnRuleMeta, fields: Vec<String>) -> Self {
        Self {
            cod_erro: meta.cod_erro.to_string(),
            message: meta.msg_erro.to_string(),
            fields,
        }
    }

    pub fn with_cod(cod: &str, message: impl Into<String>, fields: Vec<String>) -> Self {
        let message = message.into();
        Self {
            cod_erro: cod.to_string(),
            message,
            fields,
        }
    }
}

impl From<RnViolation> for ApiError {
    fn from(v: RnViolation) -> Self {
        ApiError::ValidationRn(ValidationRnError {
            error: "validation",
            cod_erro: v.cod_erro,
            message: v.message,
            fields: v.fields,
        })
    }
}

pub fn violation_for_cod(cod: &str, fields: Vec<String>) -> RnViolation {
    if let Some(meta) = rule_by_cod(cod) {
        return RnViolation::from_meta(meta, fields);
    }
    RnViolation::with_cod(cod, format!("Regra {cod}"), fields)
}
