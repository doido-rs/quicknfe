//! Endereço nacional para XML DPS (prestador, tomador, intermediário).

use crate::services::api_error::ApiError;

use super::dps_context::only_digits;

#[derive(Debug, Clone)]
pub struct EnderecoBrasil {
    pub cep: String,
    pub logradouro: String,
    pub numero: String,
    pub bairro: String,
}

pub fn endereco_from_optional(
    cep: Option<&str>,
    logradouro: Option<&str>,
    numero: Option<&str>,
    bairro: Option<&str>,
) -> Result<EnderecoBrasil, ApiError> {
    let cep = cep.map(str::trim).filter(|s| !s.is_empty());
    let logradouro = logradouro.map(str::trim).filter(|s| !s.is_empty());
    let numero = numero.map(str::trim).filter(|s| !s.is_empty());
    let bairro = bairro.map(str::trim).filter(|s| !s.is_empty());

    let cep_digits = cep.map(only_digits).unwrap_or_default();
    if cep_digits.len() != 8 {
        return Err(ApiError::Validation(
            "RN: CEP do endereço (Brasil) deve ter 8 dígitos".into(),
        ));
    }
    let logradouro = logradouro
        .ok_or_else(|| ApiError::Validation("RN: logradouro (xLgr) é obrigatório".into()))?;
    let numero = numero
        .ok_or_else(|| ApiError::Validation("RN: número (nro) do endereço é obrigatório".into()))?;
    let bairro = bairro
        .ok_or_else(|| ApiError::Validation("RN: bairro (xBairro) é obrigatório".into()))?;

    Ok(EnderecoBrasil {
        cep: cep_digits,
        logradouro: logradouro.to_string(),
        numero: numero.to_string(),
        bairro: bairro.to_string(),
    })
}

pub fn endereco_prestador(
    cep: Option<&str>,
    logradouro: Option<&str>,
    numero: Option<&str>,
    bairro: Option<&str>,
) -> Result<EnderecoBrasil, ApiError> {
    endereco_from_optional(cep, logradouro, numero, bairro).map_err(|e| match e {
        ApiError::Validation(msg) => ApiError::Validation(format!(
            "Cadastre endereço completo no prestador (CEP, logradouro, número, bairro). {msg}"
        )),
        other => other,
    })
}
