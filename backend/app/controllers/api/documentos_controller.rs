use chrono::{DateTime, NaiveDate, Utc};
use doido::controller::controller;
use doido::model::sea_orm::prelude::Decimal;
use doido::model::sea_orm::{ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::Deserialize;

use crate::models::cst_pis_cofins::CstPisCofins;
use crate::models::documento_fiscal::{
    ActiveModel as DocActive, Column as DocColumn, Entity as DocEntity, Model as Documento,
};
use crate::models::documento_modelo::DocumentoModelo;
use crate::models::documento_status::DocumentoStatus;
use crate::models::issqn_tp_suspensao::IssqnTpSuspensao;
use crate::models::localidade_parte::LocalidadeParte;
use crate::models::reg_esp_trib_issqn::RegEspTribIssqn;
use crate::models::tp_ret_pis_cofins_csll::TpRetPisCofinsCsll;
use crate::models::trib_issqn::TribIssqn;
use crate::services::api_error::{
    db, id_param, json_body, json_with_status, require_tenant, scoped, to_response, ApiError,
};
use crate::services::documento_presentacao::{documento_detalhe, documentos_list_items};
use crate::services::dto::ParteExteriorDto;
use crate::services::documento_enqueue::{enqueue_assinar, enqueue_transmitir};
use crate::controllers::api::itens_controller::ItemForm;
use crate::models::item_documento_fiscal::Model as ItemDoc;
use crate::services::item_documento::resolve_item_tributacao;
use crate::services::nfse::{
    load_dps_context, load_dps_context_with_itens, montar_dps_xml_context,
    validar_dps_contexto_com_catalogo, DocumentoAssinarParams,
};
use doido::model::sea_orm::TransactionTrait;
use crate::services::pagination::PageResponse;
use crate::services::municipio::require_municipio;
use crate::services::pagination::{page_params_from_ctx, paginate_select};
use crate::services::tenant_scope;

#[derive(Debug, Default, Deserialize)]
pub struct TransmitirForm {
    pub certificado_id: Option<i64>,
    pub run_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Default, Deserialize)]
pub struct AssinarForm {
    pub certificado_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct DocumentoForm {
    pub empresa_id: i64,
    pub cliente_id: Option<i64>,
    pub intermediario_id: Option<i64>,
    #[serde(default = "default_tomador_localidade")]
    pub tomador_localidade: LocalidadeParte,
    #[serde(default = "default_intermediario_localidade")]
    pub intermediario_localidade: LocalidadeParte,
    pub tomador_exterior: Option<ParteExteriorDto>,
    pub intermediario_exterior: Option<ParteExteriorDto>,
    pub lote_id: Option<i64>,
    #[serde(default = "default_modelo")]
    pub modelo: DocumentoModelo,
    pub serie: String,
    pub numero_documento: i64,
    pub chave_acesso: Option<String>,
    pub codigo_verificacao: Option<String>,
    pub ambiente: i32,
    pub tipo_emissao: i32,
    #[serde(default = "default_status")]
    pub status: DocumentoStatus,
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
    #[serde(default = "default_trib_issqn")]
    pub trib_issqn: TribIssqn,
    #[serde(default = "default_reg_esp_trib_issqn")]
    pub reg_esp_trib_issqn: RegEspTribIssqn,
    #[serde(default)]
    pub issqn_exigibilidade_suspensa: bool,
    #[serde(default = "default_issqn_tp_suspensao")]
    pub issqn_tp_suspensao: IssqnTpSuspensao,
    pub issqn_n_processo_suspensao: Option<String>,
    #[serde(default)]
    pub issqn_retencao: bool,
    #[serde(default)]
    pub issqn_beneficio_municipal: bool,
    #[serde(default)]
    pub issqn_deducao_reducao_bc: bool,
    pub issqn_aliquota: Option<Decimal>,
    pub issqn_base_calculo: Option<Decimal>,
    pub issqn_valor: Option<Decimal>,
    #[serde(default = "default_cst_pis_cofins")]
    pub cst_pis_cofins: CstPisCofins,
    #[serde(default = "default_tp_ret_pis_cofins_csll")]
    pub tp_ret_pis_cofins_csll: TpRetPisCofinsCsll,
    pub valor_irrf: Option<Decimal>,
    pub valor_contrib_sociais_retidas: Option<Decimal>,
    pub valor_contrib_previdenciaria_retida: Option<Decimal>,
    pub xml_assinado: Option<String>,
    pub xml_processado: Option<String>,
    pub numero_doc_responsabilidade_tecnica: Option<String>,
    pub documento_referencia: Option<String>,
    pub informacoes_complementares: Option<String>,
    pub numero_pedido_ordem_b2b: Option<String>,
}

fn default_tomador_localidade() -> LocalidadeParte {
    LocalidadeParte::Brasil
}

fn default_intermediario_localidade() -> LocalidadeParte {
    LocalidadeParte::NaoInformado
}

fn default_modelo() -> DocumentoModelo {
    DocumentoModelo::Nfse
}

fn default_status() -> DocumentoStatus {
    DocumentoStatus::Novo
}

fn default_zero_decimal() -> Decimal {
    Decimal::ZERO
}

fn default_trib_issqn() -> TribIssqn {
    TribIssqn::OperacaoTributavel
}

fn default_reg_esp_trib_issqn() -> RegEspTribIssqn {
    RegEspTribIssqn::Nenhum
}

fn default_cst_pis_cofins() -> CstPisCofins {
    CstPisCofins::Nenhum
}

fn default_issqn_tp_suspensao() -> IssqnTpSuspensao {
    IssqnTpSuspensao::default()
}

fn default_tp_ret_pis_cofins_csll() -> TpRetPisCofinsCsll {
    TpRetPisCofinsCsll::NaoRetidos
}

fn normalizar_valores_documento(form: &mut DocumentoForm) -> Result<(), ApiError> {
    if form.valor_total_servicos_produtos <= Decimal::ZERO {
        return Err(ApiError::Validation(
            "O valor da operação/serviço prestado deve ser maior que zero.".into(),
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
        return Err(ApiError::Validation(format!("{campo} é obrigatório.")));
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
    localidade: LocalidadeParte,
    relation_id: &mut Option<i64>,
    exterior: &mut Option<ParteExteriorDto>,
    parte: &str,
) -> Result<(), ApiError> {
    match localidade {
        LocalidadeParte::Brasil => {
            *exterior = None;
        }
        LocalidadeParte::NaoInformado => {
            *relation_id = None;
            *exterior = None;
        }
        LocalidadeParte::Exterior => {
            *relation_id = None;
            let ext = exterior
                .as_ref()
                .ok_or_else(|| {
                    ApiError::Validation(format!(
                        "Preencha os dados de {parte} no exterior."
                    ))
                })?;
            validar_parte_exterior(ext, parte)?;
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
        form.tomador_localidade,
        &mut form.cliente_id,
        &mut form.tomador_exterior,
        "tomador",
    )?;
    normalizar_parte_localidade(
        form.intermediario_localidade,
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

async fn documento_xml_content(
    db: &doido::model::sea_orm::DatabaseConnection,
    doc: &Documento,
) -> Result<String, ApiError> {
    if let Some(xml) = doc
        .xml_processado
        .as_ref()
        .filter(|xml| !xml.trim().is_empty())
    {
        return Ok(xml.clone());
    }
    if let Some(xml) = doc
        .xml_assinado
        .as_ref()
        .filter(|xml| !xml.trim().is_empty())
    {
        return Ok(xml.clone());
    }
    let ctx = load_dps_context(db, doc.clone()).await?;
    validar_dps_contexto_com_catalogo(db, &ctx).await?;
    montar_dps_xml_context(&ctx)
}

fn xml_download_filename(doc: &Documento) -> String {
    if let Some(chave) = doc.chave_acesso.as_ref().filter(|c| !c.trim().is_empty()) {
        return format!("{chave}.xml");
    }
    format!(
        "nf-{}-{}-{}.xml",
        DocumentoModelo::from_db_code(doc.modelo)
            .map(|m| m.as_str())
            .unwrap_or("NFSE"),
        doc.serie,
        doc.numero_documento
    )
}

const SERIE_AUTOMATICA: &str = "AUTO";

async fn proximo_numero_automatico(
    db: &doido::model::sea_orm::DatabaseConnection,
    empresa_id: i64,
    modelo: DocumentoModelo,
) -> Result<i64, ApiError> {
    let ultimo = DocEntity::find()
        .filter(DocColumn::EmpresaId.eq(empresa_id))
        .filter(DocColumn::Modelo.eq(modelo.as_db_code()))
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
        return Err(ApiError::Validation("Informe a série da nota.".into()));
    }
    if form.numero_documento <= 0 {
        return Err(ApiError::Validation("Informe um número de documento válido.".into()));
    }
    Ok(())
}

/// Evita reatribuir número AUTO quando o client envia `numero_documento <= 0` no PATCH.
fn restaurar_numero_automatico_existente(form: &mut DocumentoForm, existing: &Documento) {
    if !form.serie_numero_automatico || form.numero_documento > 0 {
        return;
    }
    if existing.serie_numero_automatico != 1 || existing.numero_documento <= 0 {
        return;
    }
    form.numero_documento = existing.numero_documento;
    if form.serie.trim().is_empty() {
        form.serie = existing.serie.clone();
    }
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
                proximo_numero_automatico(db, form.empresa_id, form.modelo).await?;
        }
    }
    Ok(())
}

fn apply_form(record: &mut DocActive, form: &DocumentoForm) -> Result<(), ApiError> {
    record.empresa_id = Set(form.empresa_id);
    record.cliente_id = Set(form.cliente_id);
    record.intermediario_id = Set(form.intermediario_id);
    record.tomador_localidade = Set(form.tomador_localidade.as_db_code());
    record.intermediario_localidade = Set(form.intermediario_localidade.as_db_code());
    record.tomador_exterior = Set(encode_parte_exterior(&form.tomador_exterior)?);
    record.intermediario_exterior = Set(encode_parte_exterior(&form.intermediario_exterior)?);
    record.lote_id = Set(form.lote_id);
    record.modelo = Set(form.modelo.as_db_code());
    record.serie = Set(form.serie.clone());
    record.numero_documento = Set(form.numero_documento);
    record.chave_acesso = Set(form.chave_acesso.clone());
    record.codigo_verificacao = Set(form.codigo_verificacao.clone());
    record.ambiente = Set(form.ambiente);
    record.tipo_emissao = Set(form.tipo_emissao);
    record.status = Set(form.status.as_db_code());
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
    record.trib_issqn = Set(form.trib_issqn.as_db_code());
    record.reg_esp_trib_issqn = Set(form.reg_esp_trib_issqn.as_db_code());
    record.issqn_exigibilidade_suspensa = Set(i8::from(form.issqn_exigibilidade_suspensa));
    record.issqn_tp_suspensao = Set(form.issqn_tp_suspensao.as_db_code());
    record.issqn_n_processo_suspensao = Set(form.issqn_n_processo_suspensao.clone());
    record.issqn_retencao = Set(i8::from(form.issqn_retencao));
    record.issqn_beneficio_municipal = Set(i8::from(form.issqn_beneficio_municipal));
    record.issqn_deducao_reducao_bc = Set(i8::from(form.issqn_deducao_reducao_bc));
    record.issqn_aliquota = Set(form.issqn_aliquota);
    record.issqn_base_calculo = Set(form.issqn_base_calculo);
    record.issqn_valor = Set(form.issqn_valor);
    record.cst_pis_cofins = Set(form.cst_pis_cofins.as_db_code());
    record.tp_ret_pis_cofins_csll = Set(form.tp_ret_pis_cofins_csll.as_db_code());
    record.valor_irrf = Set(form.valor_irrf);
    record.valor_contrib_sociais_retidas = Set(form.valor_contrib_sociais_retidas);
    record.valor_contrib_previdenciaria_retida = Set(form.valor_contrib_previdenciaria_retida);
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
                |m| m,
            )
            .await,
        ));
        let items = crate::api!(to_response(
            documentos_list_items(ctx.db(), page.items).await,
        ));
        ctx.json(PageResponse {
            items,
            next_page_token: page.next_page_token,
        })
    }

    pub async fn show(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let id = crate::api!(id_param(&ctx, "id"));
        let row = crate::api!(scoped(
            tenant_scope::documento_for_company(ctx.db(), access.company.id, id).await,
        ));
        let detalhe = crate::api!(to_response(documento_detalhe(ctx.db(), row).await));
        ctx.json(detalhe)
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
        let detalhe = crate::api!(to_response(documento_detalhe(ctx.db(), created).await));
        ctx.json(detalhe)
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
        restaurar_numero_automatico_existente(&mut form, &existing);
        crate::api!(to_response(preparar_identificacao(ctx.db(), &mut form).await));
        let mut record: DocActive = existing.into();
        crate::api!(to_response(apply_form(&mut record, &form)));
        let updated = crate::api!(db(record.update(ctx.db()).await));
        let detalhe = crate::api!(to_response(documento_detalhe(ctx.db(), updated).await));
        ctx.json(detalhe)
    }

    /// Enfileira assinatura XML-DSig da DPS (worker `cargo doido worker`).
    pub async fn assinar(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let id = crate::api!(id_param(&ctx, "id"));
        let existing = crate::api!(scoped(
            tenant_scope::documento_for_company(ctx.db(), access.company.id, id).await,
        ));
        let dps_ctx = crate::api!(to_response(
            load_dps_context(ctx.db(), existing.clone()).await,
        ));
        crate::api!(to_response(
            validar_dps_contexto_com_catalogo(ctx.db(), &dps_ctx).await,
        ));
        let form: AssinarForm = match ctx.body_json().await {
            Ok(f) => f,
            Err(_) => AssinarForm::default(),
        };
        let enqueued = crate::api!(to_response(enqueue_assinar(
            ctx.db(),
            DocumentoAssinarParams {
                company_id: access.company.id,
                documento_id: id,
                certificado_id: form.certificado_id,
            },
        )
        .await));
        json_with_status(202, enqueued)
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
        let xml = crate::api!(to_response(documento_xml_content(ctx.db(), &row).await));
        let filename = xml_download_filename(&row);
        ctx.send_data(
            xml.into_bytes(),
            "application/xml; charset=utf-8",
            Some(&filename),
        )
    }

    /// Enfileira transmissão da DPS assinada à SEFIN Nacional.
    pub async fn transmitir(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let id = crate::api!(id_param(&ctx, "id"));
        let form: TransmitirForm = match ctx.body_json().await {
            Ok(f) => f,
            Err(_) => TransmitirForm::default(),
        };
        let _doc = crate::api!(scoped(
            tenant_scope::documento_for_company(ctx.db(), access.company.id, id).await,
        ));
        let enqueued = crate::api!(to_response(
            enqueue_transmitir(
                ctx.db(),
                access.company.id,
                id,
                form.certificado_id,
                form.run_at,
            )
            .await,
        ));
        json_with_status(202, enqueued)
    }

    /// Jobs doido enfileirados para este documento (assinatura / transmissão).
    pub async fn jobs(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let id = crate::api!(id_param(&ctx, "id"));
        let doc = crate::api!(scoped(
            tenant_scope::documento_for_company(ctx.db(), access.company.id, id).await,
        ));
        let rows = crate::services::documento_jobs::list_jobs_for_documento(&doc);
        ctx.json(rows)
    }
}

#[cfg(test)]
mod identificacao_tests {
    use super::*;
    use chrono::Utc;
    use doido::model::sea_orm::prelude::Decimal;

    fn sample_existing_auto(numero: i64) -> Documento {
        Documento {
            id: 1,
            empresa_id: 1,
            cliente_id: None,
            intermediario_id: None,
            tomador_localidade: LocalidadeParte::NaoInformado.as_db_code(),
            intermediario_localidade: LocalidadeParte::NaoInformado.as_db_code(),
            tomador_exterior: None,
            intermediario_exterior: None,
            lote_id: None,
            modelo: DocumentoModelo::Nfse.as_db_code(),
            serie: SERIE_AUTOMATICA.into(),
            numero_documento: numero,
            chave_acesso: None,
            codigo_verificacao: None,
            ambiente: 2,
            tipo_emissao: 1,
            status: DocumentoStatus::Novo.as_db_code(),
            processamento_mensagem: None,
            data_emissao: Utc::now(),
            data_competencia: None,
            serie_numero_automatico: 1,
            municipio_emissao_id: 1,
            municipio_prestacao_id: 1,
            protocolo_autorizacao: None,
            valor_total_servicos_produtos: Decimal::from(100),
            valor_liquido: Decimal::from(100),
            valor_recebido_intermediario: None,
            desconto_incondicionado: Decimal::ZERO,
            desconto_condicionado: Decimal::ZERO,
            trib_issqn: TribIssqn::OperacaoTributavel.as_db_code(),
            reg_esp_trib_issqn: RegEspTribIssqn::Nenhum.as_db_code(),
            issqn_exigibilidade_suspensa: 0,
            issqn_tp_suspensao: IssqnTpSuspensao::default().as_db_code(),
            issqn_n_processo_suspensao: None,
            issqn_retencao: 0,
            issqn_beneficio_municipal: 0,
            issqn_deducao_reducao_bc: 0,
            issqn_aliquota: None,
            issqn_base_calculo: None,
            issqn_valor: None,
            cst_pis_cofins: CstPisCofins::Nenhum.as_db_code(),
            tp_ret_pis_cofins_csll: TpRetPisCofinsCsll::NaoRetidos.as_db_code(),
            valor_irrf: None,
            valor_contrib_sociais_retidas: None,
            valor_contrib_previdenciaria_retida: None,
            xml_assinado: None,
            xml_processado: None,
            numero_doc_responsabilidade_tecnica: None,
            documento_referencia: None,
            informacoes_complementares: None,
            numero_pedido_ordem_b2b: None,
        }
    }

    #[test]
    fn restaurar_numero_automatico_reutiliza_existente() {
        let existing = sample_existing_auto(17);
        let mut form = DocumentoForm {
            empresa_id: 1,
            cliente_id: None,
            intermediario_id: None,
            tomador_localidade: LocalidadeParte::NaoInformado,
            intermediario_localidade: LocalidadeParte::NaoInformado,
            tomador_exterior: None,
            intermediario_exterior: None,
            lote_id: None,
            modelo: DocumentoModelo::Nfse,
            serie: String::new(),
            numero_documento: 0,
            chave_acesso: None,
            codigo_verificacao: None,
            ambiente: 2,
            tipo_emissao: 1,
            status: DocumentoStatus::Novo,
            data_emissao: Utc::now(),
            data_competencia: None,
            serie_numero_automatico: true,
            municipio_emissao_id: 1,
            municipio_prestacao_id: 1,
            protocolo_autorizacao: None,
            valor_total_servicos_produtos: Decimal::from(100),
            valor_liquido: Decimal::from(100),
            valor_recebido_intermediario: None,
            desconto_incondicionado: Decimal::ZERO,
            desconto_condicionado: Decimal::ZERO,
            trib_issqn: TribIssqn::OperacaoTributavel,
            reg_esp_trib_issqn: RegEspTribIssqn::Nenhum,
            issqn_exigibilidade_suspensa: false,
            issqn_tp_suspensao: IssqnTpSuspensao::default(),
            issqn_n_processo_suspensao: None,
            issqn_retencao: false,
            issqn_beneficio_municipal: false,
            issqn_deducao_reducao_bc: false,
            issqn_aliquota: None,
            issqn_base_calculo: None,
            issqn_valor: None,
            cst_pis_cofins: CstPisCofins::Nenhum,
            tp_ret_pis_cofins_csll: TpRetPisCofinsCsll::NaoRetidos,
            valor_irrf: None,
            valor_contrib_sociais_retidas: None,
            valor_contrib_previdenciaria_retida: None,
            xml_assinado: None,
            xml_processado: None,
            numero_doc_responsabilidade_tecnica: None,
            documento_referencia: None,
            informacoes_complementares: None,
            numero_pedido_ordem_b2b: None,
        };

        restaurar_numero_automatico_existente(&mut form, &existing);
        assert_eq!(form.numero_documento, 17);
        assert_eq!(form.serie, "AUTO");
    }
}
