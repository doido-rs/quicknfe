//! Assinatura/transmissão NFSe (sync service + enqueue para jobs).

use chrono::Utc;
use doido::model::sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set,
};
use serde::Serialize;

use crate::models::certificado_digital::{Column as CertColumn, Entity as CertEntity};
use crate::models::documento_fiscal::{
    ActiveModel as DocActive, Entity as DocEntity, Model as Documento,
};
use crate::services::api_error::ApiError;
use crate::services::dto::decode_certificado_senha;
use crate::services::tenant_scope;

use super::{
    assinar_dps_xml_rascunho, load_dps_context, montar_dps_xml_context,
    validar_dps_contexto_com_catalogo,
};

pub mod documento_status {
    pub const ASSINANDO: &str = "assinando";
    pub const ASSINADO: &str = "assinado";
    pub const TRANSMITINDO: &str = "transmitindo";
    pub const AUTORIZADA: &str = "autorizada";
    pub const ERRO_ASSINATURA: &str = "erro_assinatura";
    pub const ERRO_TRANSMISSAO: &str = "erro_transmissao";
}

#[derive(Debug, Clone, Serialize)]
pub struct DocumentoEnqueueResponse {
    pub documento_id: i64,
    pub status: String,
    pub job_id: Option<String>,
    pub processamento_mensagem: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DocumentoAssinarParams {
    pub company_id: i64,
    pub documento_id: i64,
    pub certificado_id: Option<i64>,
}

pub async fn certificado_para_assinatura(
    db: &doido::model::sea_orm::DatabaseConnection,
    empresa_id: i64,
    certificado_id: Option<i64>,
) -> Result<crate::models::certificado_digital::Model, ApiError> {
    let cert = if let Some(id) = certificado_id {
        tenant_scope::certificado_for_tenant(db, empresa_id, id).await?
    } else {
        CertEntity::find()
            .filter(CertColumn::EmpresaId.eq(empresa_id))
            .order_by_desc(CertColumn::CriadoEm)
            .one(db)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?
            .ok_or_else(|| {
                ApiError::Validation(
                    "Cadastre um certificado digital A1 (PFX) no prestador antes de assinar a nota."
                        .into(),
                )
            })?
    };
    if cert.data_validade < Utc::now() {
        return Err(ApiError::Validation(
            "Certificado digital expirado. Envie um certificado A1 válido no prestador.".into(),
        ));
    }
    Ok(cert)
}

pub async fn assinar_documento_nfse(
    db: &doido::model::sea_orm::DatabaseConnection,
    doc: Documento,
    certificado_id: Option<i64>,
) -> Result<Documento, ApiError> {
    let dps_ctx = load_dps_context(db, doc.clone()).await?;
    validar_dps_contexto_com_catalogo(db, &dps_ctx).await?;
    // Sempre remonta a DPS a partir dos dados atuais (leiaute/namespace); não reutiliza xml_assinado.
    let base_xml = montar_dps_xml_context(&dps_ctx)?;
    let cert = certificado_para_assinatura(db, doc.empresa_id, certificado_id).await?;
    let caminho = cert
        .caminho_arquivo
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| {
            ApiError::Validation(
                "Arquivo do certificado não encontrado. Reenvie o certificado A1 no prestador."
                    .into(),
            )
        })?;
    let senha_owned = decode_certificado_senha(&cert.senha_encriptada);
    let signed = assinar_dps_xml_rascunho(
        &base_xml,
        Some(caminho),
        Some(&senha_owned),
        Some(&dps_ctx.prestador.cpf_cnpj),
    )?;
    let mut record: DocActive = doc.into();
    record.xml_assinado = Set(Some(signed));
    record.status = Set(documento_status::ASSINADO.into());
    record.processamento_mensagem = Set(None);
    let updated = record
        .update(db)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(updated)
}

fn reject_if_in_flight(status: &str) -> Result<(), ApiError> {
    match status {
        s if s == documento_status::ASSINANDO || s == documento_status::TRANSMITINDO => {
            Err(ApiError::Validation(
                "Nota em processamento. Aguarde a conclusão ou consulte o status.".into(),
            ))
        }
        _ => Ok(()),
    }
}

pub fn reject_in_flight(status: &str) -> Result<(), ApiError> {
    reject_if_in_flight(status)
}

pub async fn marcar_erro_assinatura(
    db: &doido::model::sea_orm::DatabaseConnection,
    documento_id: i64,
    msg: &str,
) -> Result<(), ApiError> {
    let doc = DocEntity::find_by_id(documento_id)
        .one(db)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::Validation("documento não encontrado".into()))?;
    let mut record: DocActive = doc.into();
    record.status = Set(documento_status::ERRO_ASSINATURA.into());
    record.processamento_mensagem = Set(Some(msg.to_string()));
    record
        .update(db)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(())
}

pub async fn marcar_erro_transmissao(
    db: &doido::model::sea_orm::DatabaseConnection,
    documento_id: i64,
    msg: &str,
) -> Result<(), ApiError> {
    let doc = DocEntity::find_by_id(documento_id)
        .one(db)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::Validation("documento não encontrado".into()))?;
    let mut record: DocActive = doc.into();
    record.status = Set(documento_status::ERRO_TRANSMISSAO.into());
    record.processamento_mensagem = Set(Some(msg.to_string()));
    record
        .update(db)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(())
}
