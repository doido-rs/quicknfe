use std::fs;

use chrono::{DateTime, NaiveDate, Utc};
use doido::controller::controller;
use doido::model::sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::Deserialize;

use crate::models::certificado_digital::{
    ActiveModel as CertActive, Column as CertColumn, Entity as CertEntity, Model as Certificado,
};
use crate::services::api_error::{db, id_param, json_body, require_tenant, scoped, to_response, ApiError};
use crate::services::certificado_arquivo::{
    decode_certificado_base64, gravar_certificado_pfx_bytes, tentar_remover_arquivo_certificado,
};
use crate::services::certificado_pfx::verificar_pfx;
use crate::services::certificado_tipo::validar_certificado_tipo;
use crate::services::dto::{encode_certificado_senha, CertificadoDto};
use crate::services::pagination::{page_params_from_ctx, paginate_select};
use crate::services::tenant_scope;

#[derive(Debug, Deserialize)]
pub struct CertificadoForm {
    pub tipo: String,
    pub caminho_arquivo: Option<String>,
    #[serde(default)]
    pub arquivo_base64: Option<String>,
    #[serde(default)]
    pub arquivo_nome: Option<String>,
    pub senha: Option<String>,
    #[serde(default, deserialize_with = "deserialize_data_validade_opt")]
    pub data_validade: Option<DateTime<Utc>>,
}

fn deserialize_data_validade_opt<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Input {
        DateTime(DateTime<Utc>),
        Date(NaiveDate),
    }

    let opt = Option::<Input>::deserialize(deserializer)?;
    match opt {
        None => Ok(None),
        Some(Input::DateTime(dt)) => Ok(Some(dt)),
        Some(Input::Date(d)) => d
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| D::Error::custom("data_validade inválida"))
            .map(|ndt| Some(ndt.and_utc())),
    }
}

fn has_new_upload(form: &CertificadoForm) -> bool {
    form.arquivo_base64
        .as_ref()
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false)
}

fn resolve_data_validade_create(
    form: &CertificadoForm,
    prestador_id: i64,
    senha: &str,
) -> Result<(String, DateTime<Utc>), ApiError> {
    let b64 = form
        .arquivo_base64
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| ApiError::Validation("arquivo do certificado é obrigatório".into()))?;
    let nome = form
        .arquivo_nome
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or("certificado.pfx");
    let bytes = decode_certificado_base64(b64)?;
    let validade = verificar_pfx(&bytes, senha)?;
    let caminho = gravar_certificado_pfx_bytes(prestador_id, nome, &bytes)?;
    Ok((caminho, validade))
}

fn resolve_data_validade_update(
    form: &CertificadoForm,
    existing: &Certificado,
    prestador_id: i64,
) -> Result<(Option<String>, DateTime<Utc>), ApiError> {
    let new_upload = has_new_upload(form);
    let new_senha = form.senha.as_ref().filter(|s| !s.is_empty());

    if new_upload {
        let senha = new_senha.ok_or_else(|| {
            ApiError::Validation(
                "senha é obrigatória ao enviar novo arquivo de certificado".into(),
            )
        })?;
        let b64 = form.arquivo_base64.as_ref().unwrap().trim();
        let nome = form
            .arquivo_nome
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or("certificado.pfx");
        let bytes = decode_certificado_base64(b64)?;
        let validade = verificar_pfx(&bytes, senha)?;
        if let Some(old) = existing.caminho_arquivo.as_deref() {
            tentar_remover_arquivo_certificado(old);
        }
        let caminho = gravar_certificado_pfx_bytes(prestador_id, nome, &bytes)?;
        return Ok((Some(caminho), validade));
    }

    if let Some(senha) = new_senha {
        let caminho = existing.caminho_arquivo.as_deref().ok_or_else(|| {
            ApiError::Validation("certificado sem arquivo para validar senha".into())
        })?;
        let bytes = fs::read(caminho).map_err(|e| {
            ApiError::Validation(format!("não foi possível ler certificado: {e}"))
        })?;
        let validade = verificar_pfx(&bytes, senha)?;
        return Ok((None, validade));
    }

    Ok((None, existing.data_validade))
}

pub struct ApiCertificadosController;

#[controller]
impl ApiCertificadosController {
    pub async fn index(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let prestador_id = crate::api!(id_param(&ctx, "prestador_id"));
        crate::api!(scoped(
            tenant_scope::prestador_for_company(ctx.db(), access.company.id, prestador_id).await,
        ));
        let params = crate::api!(to_response(page_params_from_ctx(&ctx)));
        let mut select = CertEntity::find().filter(CertColumn::EmpresaId.eq(prestador_id));
        if let Some(ref q) = params.q {
            select = select.filter(CertColumn::Tipo.contains(q));
        }
        let page = crate::api!(db(
            paginate_select(
                ctx.db(),
                select,
                CertColumn::Id,
                params,
                |m| m.id,
                CertificadoDto::from,
            )
            .await,
        ));
        ctx.json(page)
    }

    pub async fn show(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let prestador_id = crate::api!(id_param(&ctx, "prestador_id"));
        crate::api!(scoped(
            tenant_scope::prestador_for_company(ctx.db(), access.company.id, prestador_id).await,
        ));
        let id = crate::api!(id_param(&ctx, "id"));
        let row = crate::api!(scoped(
            tenant_scope::certificado_for_tenant(ctx.db(), prestador_id, id).await,
        ));
        ctx.json(CertificadoDto::from(row))
    }

    pub async fn create(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let prestador_id = crate::api!(id_param(&ctx, "prestador_id"));
        crate::api!(scoped(
            tenant_scope::prestador_for_company(ctx.db(), access.company.id, prestador_id).await,
        ));
        let form: CertificadoForm = crate::api!(json_body(ctx).await);
        crate::api!(to_response(validar_certificado_tipo(&form.tipo)));
        let senha = crate::api!(to_response(
            form.senha
                .as_ref()
                .filter(|s| !s.is_empty())
                .ok_or(ApiError::Validation("senha is required".into())),
        ));
        let (caminho, data_validade) =
            crate::api!(to_response(resolve_data_validade_create(&form, prestador_id, senha)));
        let created = crate::api!(db(
            CertActive {
                empresa_id: Set(prestador_id),
                tipo: Set(form.tipo),
                caminho_arquivo: Set(Some(caminho)),
                senha_encriptada: Set(encode_certificado_senha(senha)),
                data_validade: Set(data_validade),
                criado_em: Set(Utc::now()),
                ..Default::default()
            }
            .insert(ctx.db())
            .await,
        ));
        ctx.json(CertificadoDto::from(created))
    }

    pub async fn update(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let prestador_id = crate::api!(id_param(&ctx, "prestador_id"));
        crate::api!(scoped(
            tenant_scope::prestador_for_company(ctx.db(), access.company.id, prestador_id).await,
        ));
        let id = crate::api!(id_param(&ctx, "id"));
        let existing = crate::api!(scoped(
            tenant_scope::certificado_for_tenant(ctx.db(), prestador_id, id).await,
        ));
        let form: CertificadoForm = crate::api!(json_body(ctx).await);
        crate::api!(to_response(validar_certificado_tipo(&form.tipo)));
        let (caminho, data_validade) = crate::api!(to_response(resolve_data_validade_update(
            &form,
            &existing,
            prestador_id,
        )));
        let mut record: CertActive = existing.into();
        record.tipo = Set(form.tipo);
        if let Some(c) = caminho {
            record.caminho_arquivo = Set(Some(c));
        } else if form.caminho_arquivo.is_some() {
            record.caminho_arquivo = Set(form.caminho_arquivo);
        }
        record.data_validade = Set(data_validade);
        if let Some(senha) = form.senha.filter(|s| !s.is_empty()) {
            record.senha_encriptada = Set(encode_certificado_senha(&senha));
        }
        let updated = crate::api!(db(record.update(ctx.db()).await));
        ctx.json(CertificadoDto::from(updated))
    }

    pub async fn destroy(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let prestador_id = crate::api!(id_param(&ctx, "prestador_id"));
        crate::api!(scoped(
            tenant_scope::prestador_for_company(ctx.db(), access.company.id, prestador_id).await,
        ));
        let id = crate::api!(id_param(&ctx, "id"));
        let existing = crate::api!(scoped(
            tenant_scope::certificado_for_tenant(ctx.db(), prestador_id, id).await,
        ));
        if let Some(caminho) = existing.caminho_arquivo.as_deref() {
            tentar_remover_arquivo_certificado(caminho);
        }
        crate::api!(db(CertEntity::delete_by_id(id).exec(ctx.db()).await));
        ctx.status(204)
    }
}

#[cfg(test)]
mod tests {
    use super::CertificadoForm;

    #[test]
    fn form_json_sem_data_validade() {
        let json = r#"{"tipo":"A1","senha":"secret","arquivo_nome":"a.pfx","arquivo_base64":"YQ=="}"#;
        let form: CertificadoForm = serde_json::from_str(json).expect("parse");
        assert_eq!(form.tipo, "A1");
        assert!(form.data_validade.is_none());
    }

    #[test]
    fn form_json_ignora_data_validade_opcional() {
        let json = r#"{"tipo":"A1","senha":"secret","arquivo_nome":"a.pfx","arquivo_base64":"YQ==","data_validade":"2027-08-15"}"#;
        let form: CertificadoForm = serde_json::from_str(json).expect("parse");
        assert!(form.data_validade.is_some());
    }
}
