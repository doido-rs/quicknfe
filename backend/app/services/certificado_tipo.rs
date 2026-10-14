use crate::services::api_error::ApiError;

pub const CERTIFICADO_TIPOS: [&str; 4] = ["A1", "A3", "PFX", "PEM"];

pub fn validar_certificado_tipo(tipo: &str) -> Result<(), ApiError> {
    let t = tipo.trim();
    if CERTIFICADO_TIPOS.contains(&t) {
        Ok(())
    } else {
        Err(ApiError::Validation(format!(
            "tipo must be one of: {}",
            CERTIFICADO_TIPOS.join(", ")
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aceita_a1_rejeita_invalido() {
        validar_certificado_tipo("A1").unwrap();
        assert!(validar_certificado_tipo("INVALID").is_err());
    }
}
