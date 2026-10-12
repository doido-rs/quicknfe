use chrono::{DateTime, NaiveDate, Utc};
use doido::controller::controller;
use doido::model::sea_orm::prelude::Decimal;
use doido::model::sea_orm::{ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::Deserialize;

use crate::models::documento_fiscal::{
    ActiveModel as DocActive, Column as DocColumn, Entity as DocEntity, Model as Documento,
};
use crate::services::api_error::{db, id_param, json_body, require_tenant, scoped, to_response, ApiError};
use crate::services::dto::{
    DocumentoDto, DocumentoListDto, ParteExteriorDto, TransmitirDocumentoResponse,
};
use crate::services::nfse::{transmitir_documento, TransmitirParams};
use crate::services::municipio::require_municipio;
use crate::services::pagination::{page_params_from_ctx, paginate_select};
use crate::services::tenant_scope;

#[derive(Debug, Default, Deserialize)]
pub struct TransmitirForm {
    pub certificado_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct DocumentoForm {
    pub empresa_id: i64,
    pub cliente_id: Option<i64>,
    pub intermediario_id: Option<i64>,
    #[serde(default = "default_tomador_localidade")]
    pub tomador_localidade: String,
    #[serde(default = "default_intermediario_localidade")]
    pub intermediario_localidade: String,
    pub tomador_exterior: Option<ParteExteriorDto>,
    pub intermediario_exterior: Option<ParteExteriorDto>,
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
    #[serde(default)]
    pub serie_numero_automatico: bool,
    pub municipio_emissao_id: i64,
    pub municipio_prestacao_id: i64,
    pub protocolo_autorizacao: Option<String>,
    pub valor_total_servicos_produtos: Decimal,
    pub valor_liquido: Decimal,
    pub valor_recebido_intermediario: Option<Decimal>,
    #[serde(default = "default_zero_decimal")]
    pub desconto_incondicionado: Decimal,
    #[serde(default = "default_zero_decimal")]
    pub desconto_condicionado: Decimal,
    pub xml_assinado: Option<String>,
    pub xml_processado: Option<String>,
    pub numero_doc_responsabilidade_tecnica: Option<String>,
    pub documento_referencia: Option<String>,
    pub informacoes_complementares: Option<String>,
    pub numero_pedido_ordem_b2b: Option<String>,
}

fn default_tomador_localidade() -> String {
    "brasil".into()
}

fn default_intermediario_localidade() -> String {
    "nao_informado".into()
}

fn default_zero_decimal() -> Decimal {
    Decimal::ZERO
}

fn normalizar_valores_documento(form: &mut DocumentoForm) -> Result<(), ApiError> {
    if form.valor_total_servicos_produtos <= Decimal::ZERO {
        return Err(ApiError::Validation(
            "valor da operação/serviço prestado must be positive".into(),
        ));
    }
    let liquido = form.valor_total_servicos_produtos
        - form.desconto_incondicionado
        - form.desconto_condicionado;
    form.valor_liquido = if liquido < Decimal::ZERO {
        Decimal::ZERO
    } else {
        liquido
    };
    Ok(())
}

fn validar_campo_obrigatorio(valor: &str, campo: &str) -> Result<(), ApiError> {
    if valor.trim().is_empty() {
        return Err(ApiError::Validation(format!("{campo} is required")));
    }
    Ok(())
}

fn validar_parte_exterior(ext: &ParteExteriorDto, parte: &str) -> Result<(), ApiError> {
    validar_campo_obrigatorio(&ext.razao_social, &format!("{parte}.razao_social"))?;
    validar_campo_obrigatorio(&ext.logradouro, &format!("{parte}.logradouro"))?;
    validar_campo_obrigatorio(&ext.numero, &format!("{parte}.numero"))?;
    validar_campo_obrigatorio(&ext.bairro, &format!("{parte}.bairro"))?;
    validar_campo_obrigatorio(&ext.cidade, &format!("{parte}.cidade"))?;
    validar_campo_obrigatorio(&ext.codigo_postal, &format!("{parte}.codigo_postal"))?;
    validar_campo_obrigatorio(&ext.estado_regiao, &format!("{parte}.estado_regiao"))?;
    validar_campo_obrigatorio(&ext.pais, &format!("{parte}.pais"))?;
    Ok(())
}

fn normalizar_parte_localidade(
    localidade: &str,
    relation_id: &mut Option<i64>,
    exterior: &mut Option<ParteExteriorDto>,
    parte: &str,
) -> Result<(), ApiError> {
    match localidade {
        "brasil" => {
            *exterior = None;
        }
        "nao_informado" => {
            *relation_id = None;
            *exterior = None;
        }
        "exterior" => {
            *relation_id = None;
            let ext = exterior
                .as_ref()
                .ok_or_else(|| ApiError::Validation(format!("{parte}_exterior is required")))?;
            validar_parte_exterior(ext, parte)?;
        }
        other => {
            return Err(ApiError::Validation(format!(
                "invalid {parte}_localidade: {other}"
            )));
        }
    }
    Ok(())
}

fn encode_parte_exterior(ext: &Option<ParteExteriorDto>) -> Result<Option<String>, ApiError> {
    match ext {
        None => Ok(None),
        Some(parte) => serde_json::to_string(parte)
            .map(Some)
            .map_err(|e| ApiError::Internal(e.to_string())),
    }
}

fn normalizar_partes(form: &mut DocumentoForm) -> Result<(), ApiError> {
    normalizar_parte_localidade(
        &form.tomador_localidade,
        &mut form.cliente_id,
        &mut form.tomador_exterior,
        "tomador",
    )?;
    normalizar_parte_localidade(
        &form.intermediario_localidade,
        &mut form.intermediario_id,
        &mut form.intermediario_exterior,
        "intermediario",
    )?;
    Ok(())
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
    require_municipio(db, form.municipio_emissao_id).await?;
    require_municipio(db, form.municipio_prestacao_id).await?;
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

const SERIE_AUTOMATICA: &str = "AUTO";

async fn proximo_numero_automatico(
    db: &doido::model::sea_orm::DatabaseConnection,
    empresa_id: i64,
    modelo: &str,
) -> Result<i64, ApiError> {
    let ultimo = DocEntity::find()
        .filter(DocColumn::EmpresaId.eq(empresa_id))
        .filter(DocColumn::Modelo.eq(modelo))
        .filter(DocColumn::Serie.eq(SERIE_AUTOMATICA))
        .order_by_desc(DocColumn::NumeroDocumento)
        .one(db)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(ultimo.map(|d| d.numero_documento + 1).unwrap_or(1))
}

fn validar_identificacao(form: &DocumentoForm) -> Result<(), ApiError> {
    if form.serie_numero_automatico {
        return Ok(());
    }
    if form.serie.trim().is_empty() {
        return Err(ApiError::Validation("serie is required".into()));
    }
    if form.numero_documento <= 0 {
        return Err(ApiError::Validation("numero_documento must be positive".into()));
    }
    Ok(())
}

async fn preparar_identificacao(
    db: &doido::model::sea_orm::DatabaseConnection,
    form: &mut DocumentoForm,
) -> Result<(), ApiError> {
    validar_identificacao(form)?;
    if form.serie_numero_automatico {
        form.serie = SERIE_AUTOMATICA.into();
        if form.numero_documento <= 0 {
            form.numero_documento =
                proximo_numero_automatico(db, form.empresa_id, &form.modelo).await?;
        }
    }
    Ok(())
}

fn apply_form(record: &mut DocActive, form: &DocumentoForm) -> Result<(), ApiError> {
    record.empresa_id = Set(form.empresa_id);
    record.cliente_id = Set(form.cliente_id);
    record.intermediario_id = Set(form.intermediario_id);
    record.tomador_localidade = Set(form.tomador_localidade.clone());
    record.intermediario_localidade = Set(form.intermediario_localidade.clone());
    record.tomador_exterior = Set(encode_parte_exterior(&form.tomador_exterior)?);
    record.intermediario_exterior = Set(encode_parte_exterior(&form.intermediario_exterior)?);
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
    record.serie_numero_automatico = Set(i8::from(form.serie_numero_automatico));
    record.municipio_emissao_id = Set(form.municipio_emissao_id);
    record.municipio_prestacao_id = Set(form.municipio_prestacao_id);
    record.protocolo_autorizacao = Set(form.protocolo_autorizacao.clone());
    record.valor_total_servicos_produtos = Set(form.valor_total_servicos_produtos);
    record.valor_liquido = Set(form.valor_liquido);
    record.valor_recebido_intermediario = Set(form.valor_recebido_intermediario);
    record.desconto_incondicionado = Set(form.desconto_incondicionado);
    record.desconto_condicionado = Set(form.desconto_condicionado);
    record.xml_assinado = Set(form.xml_assinado.clone());
    record.xml_processado = Set(form.xml_processado.clone());
    record.numero_doc_responsabilidade_tecnica =
        Set(form.numero_doc_responsabilidade_tecnica.clone());
    record.documento_referencia = Set(form.documento_referencia.clone());
    record.informacoes_complementares = Set(form.informacoes_complementares.clone());
    record.numero_pedido_ordem_b2b = Set(form.numero_pedido_ordem_b2b.clone());
    Ok(())
}

pub struct ApiDocumentosController;

#[controller]
impl ApiDocumentosController {
    pub async fn index(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let params = crate::api!(to_response(page_params_from_ctx(&ctx)));
        let empresa_ids = crate::api!(to_response(
            tenant_scope::empresa_ids_for_company(ctx.db(), access.company.id).await,
        ));
        let mut select = DocEntity::find().filter(DocColumn::EmpresaId.is_in(empresa_ids));
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
        let row = crate::api!(scoped(
            tenant_scope::documento_for_company(ctx.db(), access.company.id, id).await,
        ));
        ctx.json(DocumentoDto::from(row))
    }

    pub async fn create(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let mut form: DocumentoForm = crate::api!(json_body(ctx).await);
        crate::api!(to_response(
            tenant_scope::prestador_for_company(ctx.db(), access.company.id, form.empresa_id).await,
        ));
        crate::api!(to_response(normalizar_partes(&mut form)));
        crate::api!(to_response(validate_refs(ctx.db(), form.empresa_id, &form).await));
        crate::api!(to_response(normalizar_valores_documento(&mut form)));
        crate::api!(to_response(preparar_identificacao(ctx.db(), &mut form).await));
        let mut record = DocActive {
            ..Default::default()
        };
        crate::api!(to_response(apply_form(&mut record, &form)));
        let created = crate::api!(db(record.insert(ctx.db()).await));
        ctx.json(DocumentoDto::from(created))
    }

    pub async fn update(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let id = crate::api!(id_param(&ctx, "id"));
        let existing = crate::api!(scoped(
            tenant_scope::documento_for_company(ctx.db(), access.company.id, id).await,
        ));
        let mut form: DocumentoForm = crate::api!(json_body(ctx).await);
        crate::api!(to_response(
            tenant_scope::prestador_for_company(ctx.db(), access.company.id, form.empresa_id).await,
        ));
        crate::api!(to_response(normalizar_partes(&mut form)));
        crate::api!(to_response(validate_refs(ctx.db(), form.empresa_id, &form).await));
        crate::api!(to_response(normalizar_valores_documento(&mut form)));
        crate::api!(to_response(preparar_identificacao(ctx.db(), &mut form).await));
        let mut record: DocActive = existing.into();
        crate::api!(to_response(apply_form(&mut record, &form)));
        let updated = crate::api!(db(record.update(ctx.db()).await));
        ctx.json(DocumentoDto::from(updated))
    }

    pub async fn destroy(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let id = crate::api!(id_param(&ctx, "id"));
        crate::api!(scoped(
            tenant_scope::documento_for_company(ctx.db(), access.company.id, id).await,
        ));
        crate::api!(db(DocEntity::delete_by_id(id).exec(ctx.db()).await));
        ctx.status(204)
    }

    /// Baixa o XML da nota (`xml_processado` ou, na ausência, `xml_assinado`).
    pub async fn download_xml(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let id = crate::api!(id_param(&ctx, "id"));
        let row = crate::api!(scoped(
            tenant_scope::documento_for_company(ctx.db(), access.company.id, id).await,
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
        let doc = crate::api!(scoped(
            tenant_scope::documento_for_company(ctx.db(), access.company.id, id).await,
        ));
        let outcome = crate::api!(to_response(
            transmitir_documento(
                ctx.db(),
                TransmitirParams {
                    empresa_id: doc.empresa_id,
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
