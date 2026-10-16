use chrono::Utc;
use doido::model::sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set,
};

use super::ambiente::AmbienteNfse;
use super::cliente::{enviar_dps_assinada, CredenciaisMtls};
use super::dps_context::load_dps_context;
use super::resposta::ResultadoTransmissao;
use super::rn_validacao::validar_dps_contexto_com_catalogo;
use crate::models::certificado_digital::{Column as CertColumn, Entity as CertEntity};
use crate::models::documento_fiscal::ActiveModel as DocActive;
use crate::models::log_mensagem_retorno::ActiveModel as LogActive;
use crate::services::api_error::ApiError;
use crate::services::dto::decode_certificado_senha;
use crate::services::tenant_scope;

pub struct TransmitirParams {
    pub empresa_id: i64,
    pub documento_id: i64,
    pub certificado_id: Option<i64>,
}

pub struct TransmitirOutcome {
    pub resultado: ResultadoTransmissao,
    pub documento: crate::models::documento_fiscal::Model,
}

async fn certificado_para_envio(
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
                    "nenhum certificado digital cadastrado para a empresa".into(),
                )
            })?
    };

    if cert.data_validade < Utc::now() {
        return Err(ApiError::Validation(format!(
            "certificado id {} está vencido",
            cert.id
        )));
    }

    let caminho = cert.caminho_arquivo.clone().ok_or_else(|| {
        ApiError::Validation(format!(
            "certificado id {} não possui caminho_arquivo configurado",
            cert.id
        ))
    })?;

    if !std::path::Path::new(&caminho).is_file() {
        return Err(ApiError::Validation(format!(
            "arquivo do certificado não encontrado: {caminho}"
        )));
    }

    Ok(cert)
}

pub async fn transmitir_documento(
    db: &doido::model::sea_orm::DatabaseConnection,
    params: TransmitirParams,
) -> Result<TransmitirOutcome, ApiError> {
    let mut documento =
        tenant_scope::documento_for_tenant(db, params.empresa_id, params.documento_id).await?;

    let xml = documento.xml_assinado.clone().ok_or_else(|| {
        ApiError::Validation(
            "documento sem xml_assinado — gere e assine a DPS antes de transmitir".into(),
        )
    })?;

    if documento.status.eq_ignore_ascii_case("autorizada") {
        return Err(ApiError::Validation(
            "documento já está autorizado; transmissão não permitida".into(),
        ));
    }

    let dps_ctx = load_dps_context(db, documento.clone()).await?;
    validar_dps_contexto_com_catalogo(db, &dps_ctx).await?;

    let ambiente = AmbienteNfse::from_codigo(documento.ambiente)?;
    let cert = certificado_para_envio(db, params.empresa_id, params.certificado_id).await?;

    let caminho_pfx = cert.caminho_arquivo.clone().expect("validado acima");
    let credenciais = CredenciaisMtls {
        caminho_pfx,
        senha_pfx: decode_certificado_senha(&cert.senha_encriptada),
    };

    let resultado = enviar_dps_assinada(&xml, ambiente, &credenciais).await?;

    persistir_resultado(db, &mut documento, &resultado).await?;

    Ok(TransmitirOutcome {
        resultado,
        documento,
    })
}

async fn persistir_resultado(
    db: &doido::model::sea_orm::DatabaseConnection,
    documento: &mut crate::models::documento_fiscal::Model,
    resultado: &ResultadoTransmissao,
) -> Result<(), ApiError> {
    let mut active: DocActive = documento.clone().into();

    if resultado.sucesso {
        active.status = Set("autorizada".into());
        if let Some(chave) = &resultado.chave_acesso {
            active.chave_acesso = Set(Some(chave.clone()));
        }
        if let Some(xml) = &resultado.nfse_xml {
            active.xml_processado = Set(Some(xml.clone()));
        }
    } else {
        active.status = Set("rejeitada".into());
    }

    let updated = active
        .update(db)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    *documento = updated;

    let data_registro = resultado
        .data_hora_processamento
        .unwrap_or_else(Utc::now);

    for erro in &resultado.erros {
        let (codigo_erro, mensagem_retorno, correcao_sugerida) = erro.campos_log();
        LogActive {
            documento_fiscal_id: Set(Some(documento.id)),
            lote_id: Set(documento.lote_id),
            codigo_erro: Set(codigo_erro),
            mensagem_retorno: Set(mensagem_retorno),
            correcao_sugerida: Set(correcao_sugerida),
            data_registro: Set(data_registro),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    }

    for alerta in &resultado.alertas {
        let (codigo_erro, mensagem_retorno, correcao_sugerida) = alerta.campos_log();
        LogActive {
            documento_fiscal_id: Set(Some(documento.id)),
            lote_id: Set(documento.lote_id),
            codigo_erro: Set(if codigo_erro == "SEFIN" {
                "ALERTA".into()
            } else {
                codigo_erro
            }),
            mensagem_retorno: Set(mensagem_retorno),
            correcao_sugerida: Set(correcao_sugerida),
            data_registro: Set(data_registro),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    }

    Ok(())
}
