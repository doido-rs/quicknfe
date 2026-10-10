use chrono::{DateTime, NaiveDate, Utc};
use doido::controller::controller;
use doido::model::sea_orm::prelude::Decimal;
use doido::model::sea_orm::{ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, Set};
use serde::Deserialize;

use crate::models::documento_fiscal::{
    ActiveModel as DocActive, Column as DocColumn, Entity as DocEntity, Model as Documento,
};
use crate::services::api_error::{db, id_param, json_body, require_tenant, scoped, to_response, ApiError};
use crate::services::dto::{DocumentoDto, DocumentoListDto, TransmitirDocumentoResponse};
use crate::services::nfse::{transmitir_documento, TransmitirParams};
use crate::services::pagination::{page_params_from_ctx, paginate_select};
use crate::services::tenant_scope;

#[derive(Debug, Default, Deserialize)]
pub struct TransmitirForm {
    pub certificado_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct DocumentoForm {
    pub cliente_id: Option<i64>,
    pub intermediario_id: Option<i64>,
    pub lote_id: Option<i64>,
    pub modelo: String,
    pub serie: String,
    pub numero_documento: i64,
    pub chave_acesso: Option<String>,
    pub codigo_verificacao: Option<String>,
    pub ambiente: i32,
    pub tipo_emissao: i32,
    pub status: String,
    pub data_emissao: DateTime<Utc>,
    pub data_competencia: Option<NaiveDate>,
    pub codigo_municipio_emissao: String,
    pub protocolo_autorizacao: Option<String>,
    pub valor_total_servicos_produtos: Decimal,
    pub valor_liquido: Decimal,
    pub xml_assinado: Option<String>,
    pub xml_processado: Option<String>,
}

async fn validate_refs(
    db: &doido::model::sea_orm::DatabaseConnection,
    empresa_id: i64,
    form: &DocumentoForm,
) -> Result<(), ApiError> {
    if let Some(cid) = form.cliente_id {
        tenant_scope::ensure_cliente_belongs(db, empresa_id, cid).await?;
    }
    if let Some(iid) = form.intermediario_id {
        tenant_scope::ensure_intermediario_belongs(db, empresa_id, iid).await?;
    }
    if let Some(lid) = form.lote_id {
        tenant_scope::ensure_lote_belongs(db, empresa_id, lid).await?;
    }
    Ok(())
}

fn documento_xml_content(doc: &Documento) -> Option<String> {
    doc.xml_processado
        .clone()
        .or_else(|| doc.xml_assinado.clone())
        .filter(|xml| !xml.trim().is_empty())
}

fn xml_download_filename(doc: &Documento) -> String {
    if let Some(chave) = doc.chave_acesso.as_ref().filter(|c| !c.trim().is_empty()) {
        return format!("{chave}.xml");
    }
    format!(
        "nf-{}-{}-{}.xml",
        doc.modelo, doc.serie, doc.numero_documento
    )
}

fn apply_form(record: &mut DocActive, form: &DocumentoForm, empresa_id: i64) {
    record.empresa_id = Set(empresa_id);
    record.cliente_id = Set(form.cliente_id);
    record.intermediario_id = Set(form.intermediario_id);
    record.lote_id = Set(form.lote_id);
    record.modelo = Set(form.modelo.clone());
    record.serie = Set(form.serie.clone());
    record.numero_documento = Set(form.numero_documento);
    record.chave_acesso = Set(form.chave_acesso.clone());
    record.codigo_verificacao = Set(form.codigo_verificacao.clone());
    record.ambiente = Set(form.ambiente);
    record.tipo_emissao = Set(form.tipo_emissao);
    record.status = Set(form.status.clone());
    record.data_emissao = Set(form.data_emissao);
    record.data_competencia = Set(form.data_competencia);
    record.codigo_municipio_emissao = Set(form.codigo_municipio_emissao.clone());
    record.protocolo_autorizacao = Set(form.protocolo_autorizacao.clone());
    record.valor_total_servicos_produtos = Set(form.valor_total_servicos_produtos);
    record.valor_liquido = Set(form.valor_liquido);
    record.xml_assinado = Set(form.xml_assinado.clone());
    record.xml_processado = Set(form.xml_processado.clone());
}

pub struct ApiDocumentosController;

#[controller]
impl ApiDocumentosController {
    pub async fn index(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let params = crate::api!(to_response(page_params_from_ctx(&ctx)));
        let mut select = DocEntity::find().filter(DocColumn::EmpresaId.eq(access.prestador.id));
        if let Some(ref q) = params.q {
            let mut filter = Condition::any()
                .add(DocColumn::Serie.contains(q))
                .add(DocColumn::Status.contains(q))
                .add(DocColumn::Modelo.contains(q))
                .add(DocColumn::ChaveAcesso.contains(q));
            if let Ok(num) = q.parse::<i64>() {
                filter = filter.add(DocColumn::NumeroDocumento.eq(num));
            }
            select = select.filter(filter);
        }
        let page = crate::api!(db(
            paginate_select(
                ctx.db(),
                select,
                DocColumn::Id,
                params,
                |m| m.id,
                DocumentoListDto::from,
            )
            .await,
        ));
        ctx.json(page)
    }

    pub async fn show(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let id = crate::api!(id_param(&ctx, "id"));
        let row = crate::api!(scoped(tenant_scope::documento_for_tenant(ctx.db(), access.prestador.id, id).await));
        ctx.json(DocumentoDto::from(row))
    }

    pub async fn create(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let form: DocumentoForm = crate::api!(json_body(ctx).await);
        crate::api!(to_response(validate_refs(ctx.db(), access.prestador.id, &form).await));
        let mut record = DocActive {
            ..Default::default()
        };
        apply_form(&mut record, &form, access.prestador.id);
        let created = crate::api!(db(record.insert(ctx.db()).await));
        ctx.json(DocumentoDto::from(created))
    }

    pub async fn update(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let id = crate::api!(id_param(&ctx, "id"));
        let existing = crate::api!(scoped(tenant_scope::documento_for_tenant(ctx.db(), access.prestador.id, id).await));
        let form: DocumentoForm = crate::api!(json_body(ctx).await);
        crate::api!(to_response(validate_refs(ctx.db(), access.prestador.id, &form).await));
        let mut record: DocActive = existing.into();
        apply_form(&mut record, &form, access.prestador.id);
        let updated = crate::api!(db(record.update(ctx.db()).await));
        ctx.json(DocumentoDto::from(updated))
    }

    pub async fn destroy(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let id = crate::api!(id_param(&ctx, "id"));
        crate::api!(scoped(tenant_scope::documento_for_tenant(ctx.db(), access.prestador.id, id).await));
        crate::api!(db(DocEntity::delete_by_id(id).exec(ctx.db()).await));
        ctx.status(204)
    }

    /// Baixa o XML da nota (`xml_processado` ou, na ausência, `xml_assinado`).
    pub async fn download_xml(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let id = crate::api!(id_param(&ctx, "id"));
        let row = crate::api!(scoped(
            tenant_scope::documento_for_tenant(ctx.db(), access.prestador.id, id).await,
        ));
        let xml = documento_xml_content(&row).ok_or_else(|| {
            ApiError::Validation("Esta nota fiscal ainda não possui XML armazenado.".into())
        });
        let xml = crate::api!(to_response(xml));
        let filename = xml_download_filename(&row);
        ctx.send_data(
            xml.into_bytes(),
            "application/xml; charset=utf-8",
            Some(&filename),
        )
    }

    /// Envia a DPS já assinada (`xml_assinado`) para a API SEFIN Nacional (mTLS + JSON).
    pub async fn transmitir(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let id = crate::api!(id_param(&ctx, "id"));
        let form: TransmitirForm = match ctx.body_json().await {
            Ok(f) => f,
            Err(_) => TransmitirForm::default(),
        };
        let outcome = crate::api!(to_response(
            transmitir_documento(
                ctx.db(),
                TransmitirParams {
                    empresa_id: access.prestador.id,
                    documento_id: id,
                    certificado_id: form.certificado_id,
                },
            )
            .await,
        ));
        let r = outcome.resultado;
        ctx.json(TransmitirDocumentoResponse {
            sucesso: r.sucesso,
            http_status: r.http_status,
            id_dps: r.id_dps,
            chave_acesso: r.chave_acesso,
            nfse_xml: r.nfse_xml,
            alertas: r.alertas,
            erros: r.erros,
            documento: DocumentoDto::from(outcome.documento),
        })
    }
}
