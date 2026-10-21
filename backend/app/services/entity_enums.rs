//! Map entity integer columns to domain enums (invalid DB → Internal error).

use crate::models::certificado_tipo::CertificadoTipo;
use crate::models::cst_pis_cofins::CstPisCofins;
use crate::models::date_format_preference::DateFormatPreference;
use crate::models::documento_modelo::DocumentoModelo;
use crate::models::documento_status::DocumentoStatus;
use crate::models::issqn_tp_suspensao::IssqnTpSuspensao;
use crate::models::localidade_parte::LocalidadeParte;
use crate::models::papel_usuario::PapelUsuario;
use crate::models::reg_esp_trib_issqn::RegEspTribIssqn;
use crate::models::regime_tributario::RegimeTributario;
use crate::models::status_lote::StatusLote;
use crate::models::tipo_evento::TipoEvento;
use crate::models::tipo_imposto::TipoImposto;
use crate::models::tp_ret_pis_cofins_csll::TpRetPisCofinsCsll;
use crate::models::trib_issqn::TribIssqn;
use crate::services::api_error::ApiError;

fn internal_enum(field: &str, code: i32, err: ApiError) -> ApiError {
    ApiError::Internal(format!("{field} inválido no banco ({code}): {err}"))
}

pub fn regime_from_db(code: i32) -> Result<RegimeTributario, ApiError> {
    RegimeTributario::from_db_code(code).map_err(|e| internal_enum("regime_tributario", code, e))
}

pub fn papel_from_db(code: i32) -> Result<PapelUsuario, ApiError> {
    PapelUsuario::from_db_code(code).map_err(|e| internal_enum("papel", code, e))
}

pub fn date_format_from_db(code: i32) -> Result<DateFormatPreference, ApiError> {
    DateFormatPreference::from_db_code(code).map_err(|e| internal_enum("date_format", code, e))
}

pub fn certificado_tipo_from_db(code: i32) -> Result<CertificadoTipo, ApiError> {
    CertificadoTipo::from_db_code(code).map_err(|e| internal_enum("certificado.tipo", code, e))
}

pub fn documento_status_from_db(code: i32) -> Result<DocumentoStatus, ApiError> {
    DocumentoStatus::from_db_code(code).map_err(|e| internal_enum("status", code, e))
}

pub fn localidade_from_db(code: i32) -> Result<LocalidadeParte, ApiError> {
    LocalidadeParte::from_db_code(code).map_err(|e| internal_enum("localidade", code, e))
}

pub fn documento_modelo_from_db(code: i32) -> Result<DocumentoModelo, ApiError> {
    DocumentoModelo::from_db_code(code).map_err(|e| internal_enum("modelo", code, e))
}

pub fn trib_issqn_from_db(code: i32) -> Result<TribIssqn, ApiError> {
    TribIssqn::from_db_code(code).map_err(|e| internal_enum("trib_issqn", code, e))
}

pub fn reg_esp_trib_from_db(code: i32) -> Result<RegEspTribIssqn, ApiError> {
    RegEspTribIssqn::from_db_code(code).map_err(|e| internal_enum("reg_esp_trib_issqn", code, e))
}

pub fn issqn_tp_suspensao_from_db(code: i32) -> Result<IssqnTpSuspensao, ApiError> {
    IssqnTpSuspensao::from_db_code(code).map_err(|e| internal_enum("issqn_tp_suspensao", code, e))
}

pub fn cst_pis_cofins_from_db(code: i32) -> Result<CstPisCofins, ApiError> {
    CstPisCofins::from_db_code(code).map_err(|e| internal_enum("cst_pis_cofins", code, e))
}

pub fn tp_ret_from_db(code: i32) -> Result<TpRetPisCofinsCsll, ApiError> {
    TpRetPisCofinsCsll::from_db_code(code).map_err(|e| internal_enum("tp_ret_pis_cofins_csll", code, e))
}

pub fn status_lote_from_db(code: i32) -> Result<StatusLote, ApiError> {
    StatusLote::from_db_code(code).map_err(|e| internal_enum("status_lote", code, e))
}

pub fn tipo_evento_from_db(code: i32) -> Result<TipoEvento, ApiError> {
    TipoEvento::from_db_code(code).map_err(|e| internal_enum("tipo_evento", code, e))
}

pub fn tipo_imposto_from_db(code: i32) -> Result<TipoImposto, ApiError> {
    TipoImposto::from_db_code(code).map_err(|e| internal_enum("tipo_imposto", code, e))
}
