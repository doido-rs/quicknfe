//! Execução das regras RN no save / assinar / transmitir.

use doido::model::sea_orm::ConnectionTrait;

use crate::services::api_error::ApiError;
use crate::services::nfse::dps_context::DpsContext;
use crate::services::nfse::dps_rn_catalog::ALL_RULES;
use crate::services::nfse::dps_rn_engine;
use crate::services::nfse::e0312_validacao::validar_e0312_catalogo;
use crate::services::nfse::e0539_validacao::validar_trib_issqn_incidencia_catalogo;
use crate::services::nfse::issqn_simples_nacional::validar_e0621_e0625;
use crate::services::nfse::rn::dispatch_generated::evaluate_cod;
use crate::services::nfse::rn::violations::RnViolation;

pub fn validar_rn_sync(ctx: &DpsContext) -> Result<(), ApiError> {
    legacy_rn_checks(ctx)?;
    for rule in ALL_RULES {
        if rule.requires_db_catalog {
            continue;
        }
        evaluate_cod(rule.cod_erro, ctx).map_err(ApiError::from)?;
    }
    Ok(())
}

pub async fn validar_rn_com_catalogo(
    db: &impl ConnectionTrait,
    ctx: &DpsContext,
) -> Result<(), ApiError> {
    validar_rn_sync(ctx)?;
    validar_e0312_catalogo(db, ctx).await?;
    validar_trib_issqn_incidencia_catalogo(db, ctx).await?;
    validar_e0621_e0625(ctx)?;
    Ok(())
}

fn legacy_rn_checks(ctx: &DpsContext) -> Result<(), ApiError> {
    dps_rn_engine::validar_rn_csv_incremental(ctx).map_err(|e| match e {
        ApiError::Validation(msg) => {
            if let Some(cod) = extract_cod(&msg) {
                ApiError::from(RnViolation::with_cod(cod, msg.clone(), vec![]))
            } else {
                ApiError::Validation(msg)
            }
        }
        other => other,
    })
}

fn extract_cod(msg: &str) -> Option<&str> {
    msg.split(':').next().filter(|s| s.starts_with('E') && s.len() <= 6)
}
