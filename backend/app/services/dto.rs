use chrono::{DateTime, NaiveDate, Utc};
use doido::model::sea_orm::prelude::Decimal;
use serde::{Deserialize, Serialize};

use crate::models::cna::Model as Cna;
use crate::models::certificado_digital::Model as Certificado;
use crate::models::cliente::Model as Cliente;
use crate::models::documento_fiscal::Model as Documento;
use crate::models::company::Model as Company;
use crate::models::empresa::Model as Empresa;
use crate::models::certificado_tipo::CertificadoTipo;
use crate::models::cst_pis_cofins::CstPisCofins;
use crate::models::date_format_preference::DateFormatPreference;
use crate::models::documento_modelo::DocumentoModelo;
use crate::models::documento_status::DocumentoStatus;
use crate::models::localidade_parte::LocalidadeParte;
use crate::models::papel_usuario::PapelUsuario;
use crate::models::reg_esp_trib_issqn::RegEspTribIssqn;
use crate::models::regime_tributario::RegimeTributario;
use crate::models::status_lote::StatusLote;
use crate::models::tipo_evento::TipoEvento;
use crate::models::tipo_imposto::TipoImposto;
use crate::models::tp_ret_pis_cofins_csll::TpRetPisCofinsCsll;
use crate::models::trib_issqn::TribIssqn;
use crate::services::entity_enums::{
    certificado_tipo_from_db, cst_pis_cofins_from_db, date_format_from_db, documento_modelo_from_db,
    documento_status_from_db, localidade_from_db, reg_esp_trib_from_db,
    regime_from_db, status_lote_from_db, tipo_evento_from_db, tipo_imposto_from_db,
    tp_ret_from_db, trib_issqn_from_db,
};
use crate::models::evento_documento_fiscal::Model as Evento;
use crate::models::conta_fiscal::Model as ContaFiscal;
use crate::models::intermediario::Model as Intermediario;
use crate::models::item_documento_fiscal::Model as Item;
use crate::models::log_mensagem_retorno::Model as LogRetorno;
use crate::models::lote_transmissao::Model as Lote;
use crate::models::municipio::Model as Municipio;
use crate::models::tributo_item::Model as Tributo;
use crate::models::user::Model as User;

#[derive(Serialize, Deserialize)]
pub struct UpdateMeForm {
    pub date_format: Option<DateFormatPreference>,
}

#[derive(Serialize)]
pub struct UserPublic {
    pub id: i64,
    pub email: String,
    pub date_format: DateFormatPreference,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl UserPublic {
    pub fn try_from_user(u: User) -> Result<Self, crate::services::api_error::ApiError> {
        Ok(Self {
            id: u.id,
            email: u.email,
            date_format: date_format_from_db(u.date_format)?,
            created_at: u.created_at,
            updated_at: u.updated_at,
        })
    }
}

impl From<User> for UserPublic {
    fn from(u: User) -> Self {
        Self::try_from_user(u).expect("date_format inválido no banco")
    }
}

#[derive(Serialize)]
pub struct CnaDto {
    pub id: i64,
    pub codigo: String,
    pub descricao: String,
}

impl From<Cna> for CnaDto {
    fn from(c: Cna) -> Self {
        Self {
            id: c.id,
            codigo: c.codigo,
            descricao: c.descricao,
        }
    }
}

#[derive(Serialize)]
pub struct MunicipioDto {
    pub id: i64,
    pub codigo_ibge: String,
    pub nome: String,
    pub uf: String,
}

impl From<Municipio> for MunicipioDto {
    fn from(m: Municipio) -> Self {
        MunicipioDto::from(&m)
    }
}

impl From<&Municipio> for MunicipioDto {
    fn from(m: &Municipio) -> Self {
        Self {
            id: m.id,
            codigo_ibge: m.codigo_ibge.clone(),
            nome: m.nome.clone(),
            uf: m.uf.clone(),
        }
    }
}

#[derive(Serialize)]
pub struct EmpresaPublic {
    pub id: i64,
    pub cpf_cnpj: String,
    pub razao_social: String,
    pub inscricao_municipal: Option<String>,
    pub inscricao_estadual: Option<String>,
    pub regime_tributario: RegimeTributario,
    pub municipio_id: i64,
    pub municipio: MunicipioDto,
    pub cep: Option<String>,
    pub logradouro: Option<String>,
    pub numero: Option<String>,
    pub bairro: Option<String>,
    /// `opSimpNac` que a DPS emitiria com o regime tributário atual (1–3).
    pub op_simp_nac_dps: i32,
    /// Snapshot da consulta CNC / cadastro Simples (quando disponível).
    pub op_simp_nac_cnc: Option<i32>,
    /// Indica divergência que levaria à rejeição E0160 na SEFIN.
    pub regime_tributario_requer_atualizacao: bool,
    pub criado_em: DateTime<Utc>,
}

pub async fn empresa_public(
    db: &impl doido::model::sea_orm::ConnectionTrait,
    empresa: Empresa,
) -> Result<EmpresaPublic, crate::services::api_error::ApiError> {
    let municipio = crate::services::municipio::require_municipio(db, empresa.municipio_id).await?;
    EmpresaPublic::from_parts(empresa, &municipio)
}

pub async fn empresas_public(
    db: &impl doido::model::sea_orm::ConnectionTrait,
    empresas: Vec<Empresa>,
) -> Result<Vec<EmpresaPublic>, crate::services::api_error::ApiError> {
    let municipios =
        crate::services::municipio::map_by_ids(db, empresas.iter().map(|e| e.municipio_id)).await?;
    empresas
        .into_iter()
        .map(|e| {
            let municipio = municipios.get(&e.municipio_id).ok_or_else(|| {
                crate::services::api_error::ApiError::Internal(format!(
                    "município {} ausente no catálogo",
                    e.municipio_id
                ))
            })?;
            EmpresaPublic::from_parts(e, municipio)
        })
        .collect()
}

impl EmpresaPublic {
    pub fn from_parts(
        e: Empresa,
        m: &Municipio,
    ) -> Result<Self, crate::services::api_error::ApiError> {
        use crate::services::nfse::op_simp_nac::{
            op_simp_nac_from_regime_code, regime_requer_atualizacao_para_cnc,
        };
        let op_simp_nac_dps = op_simp_nac_from_regime_code(e.regime_tributario)?.as_i32();
        let regime_tributario_requer_atualizacao = e
            .op_simp_nac_verificado
            .map(|cnc| {
                regime_requer_atualizacao_para_cnc(e.regime_tributario, cnc).unwrap_or(false)
            })
            .unwrap_or(false);
        Ok(Self {
            id: e.id,
            cpf_cnpj: e.cpf_cnpj,
            razao_social: e.razao_social,
            inscricao_municipal: e.inscricao_municipal,
            inscricao_estadual: e.inscricao_estadual,
            regime_tributario: regime_from_db(e.regime_tributario)?,
            municipio_id: e.municipio_id,
            municipio: MunicipioDto::from(m),
            cep: e.cep,
            logradouro: e.logradouro,
            numero: e.numero,
            bairro: e.bairro,
            op_simp_nac_dps,
            op_simp_nac_cnc: e.op_simp_nac_verificado,
            regime_tributario_requer_atualizacao,
            criado_em: e.criado_em,
        })
    }
}

#[derive(Serialize)]
pub struct CompanyPublic {
    pub id: i64,
    pub nome: String,
    pub criado_em: DateTime<Utc>,
}

impl From<Company> for CompanyPublic {
    fn from(c: Company) -> Self {
        Self {
            id: c.id,
            nome: c.nome,
            criado_em: c.criado_em,
        }
    }
}

#[derive(Serialize)]
pub struct CompanyMembership {
    pub company: CompanyPublic,
    pub papel: PapelUsuario,
}

#[derive(Serialize)]
pub struct MeResponse {
    pub user: UserPublic,
    pub companies: Vec<CompanyMembership>,
    pub company_ativa: Option<CompanyPublic>,
    pub prestadores: Vec<EmpresaPublic>,
    pub prestador_ativo: Option<EmpresaPublic>,
    pub papel: Option<PapelUsuario>,
}

#[derive(Serialize)]
pub struct ClienteDto {
    pub id: i64,
    pub empresa_id: i64,
    pub cpf_cnpj: String,
    pub razao_social: String,
    pub inscricao_municipal: Option<String>,
    pub inscricao_estadual: Option<String>,
    pub logradouro: Option<String>,
    pub numero: Option<String>,
    pub bairro: Option<String>,
    pub cep: Option<String>,
    pub municipio_id: Option<i64>,
    pub municipio: Option<MunicipioDto>,
    pub email: Option<String>,
    pub telefone: Option<String>,
}

impl ClienteDto {
    pub fn from_parts(c: Cliente, m: Option<&Municipio>) -> Self {
        Self {
            id: c.id,
            empresa_id: c.empresa_id,
            cpf_cnpj: c.cpf_cnpj,
            razao_social: c.razao_social,
            inscricao_municipal: c.inscricao_municipal,
            inscricao_estadual: c.inscricao_estadual,
            logradouro: c.logradouro,
            numero: c.numero,
            bairro: c.bairro,
            cep: c.cep,
            municipio_id: c.municipio_id,
            municipio: m.map(MunicipioDto::from),
            email: c.email,
            telefone: c.telefone,
        }
    }
}

#[derive(Serialize)]
pub struct IntermediarioDto {
    pub id: i64,
    pub empresa_id: i64,
    pub cpf_cnpj: String,
    pub razao_social: String,
    pub inscricao_municipal: Option<String>,
    pub email: Option<String>,
    pub telefone: Option<String>,
    pub municipio_id: Option<i64>,
    pub cep: Option<String>,
    pub logradouro: Option<String>,
    pub numero: Option<String>,
    pub bairro: Option<String>,
}

#[derive(Serialize)]
pub struct ContaFiscalDto {
    pub id: i64,
    pub empresa_id: i64,
    pub codigo: String,
    pub descricao: Option<String>,
    pub cna_id: i64,
    pub codigo_tributacao_nacional: String,
    pub codigo_tributacao_municipal: String,
}

impl From<ContaFiscal> for ContaFiscalDto {
    fn from(c: ContaFiscal) -> Self {
        Self {
            id: c.id,
            empresa_id: c.empresa_id,
            codigo: c.codigo,
            descricao: c.descricao,
            cna_id: c.cna_id,
            codigo_tributacao_nacional: c.codigo_tributacao_nacional,
            codigo_tributacao_municipal: c.codigo_tributacao_municipal,
        }
    }
}

impl From<Intermediario> for IntermediarioDto {
    fn from(i: Intermediario) -> Self {
        Self {
            id: i.id,
            empresa_id: i.empresa_id,
            cpf_cnpj: i.cpf_cnpj,
            razao_social: i.razao_social,
            inscricao_municipal: i.inscricao_municipal,
            email: i.email,
            telefone: i.telefone,
            municipio_id: i.municipio_id,
            cep: i.cep,
            logradouro: i.logradouro,
            numero: i.numero,
            bairro: i.bairro,
        }
    }
}

#[derive(Serialize)]
pub struct CertificadoDto {
    pub id: i64,
    pub empresa_id: i64,
    pub tipo: CertificadoTipo,
    pub caminho_arquivo: Option<String>,
    pub data_validade: DateTime<Utc>,
    pub criado_em: DateTime<Utc>,
}

impl From<Certificado> for CertificadoDto {
    fn from(c: Certificado) -> Self {
        Self {
            id: c.id,
            empresa_id: c.empresa_id,
            tipo: certificado_tipo_from_db(c.tipo).expect("tipo certificado inválido"),
            caminho_arquivo: c.caminho_arquivo,
            data_validade: c.data_validade,
            criado_em: c.criado_em,
        }
    }
}

#[derive(Serialize)]
pub struct LoteDto {
    pub id: i64,
    pub empresa_id: i64,
    pub numero_lote: i64,
    pub protocolo_recebimento: Option<String>,
    pub ambiente: i32,
    pub status_lote: StatusLote,
    pub data_recebimento: Option<DateTime<Utc>>,
}

impl From<Lote> for LoteDto {
    fn from(l: Lote) -> Self {
        Self {
            id: l.id,
            empresa_id: l.empresa_id,
            numero_lote: l.numero_lote,
            protocolo_recebimento: l.protocolo_recebimento,
            ambiente: l.ambiente,
            status_lote: status_lote_from_db(l.status_lote).expect("status_lote inválido"),
            data_recebimento: l.data_recebimento,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ParteExteriorDto {
    pub razao_social: String,
    pub telefone: Option<String>,
    pub email: Option<String>,
    pub logradouro: String,
    pub numero: String,
    pub complemento: Option<String>,
    pub bairro: String,
    pub cidade: String,
    pub codigo_postal: String,
    pub estado_regiao: String,
    pub pais: String,
}

fn decode_parte_exterior(raw: &Option<String>) -> Option<ParteExteriorDto> {
    raw.as_ref().and_then(|s| serde_json::from_str(s).ok())
}

#[derive(Serialize)]
pub struct DocumentoListDto {
    pub id: i64,
    pub empresa_id: i64,
    pub cliente_id: Option<i64>,
    pub intermediario_id: Option<i64>,
    pub tomador_localidade: LocalidadeParte,
    pub intermediario_localidade: LocalidadeParte,
    pub tomador_exterior: Option<ParteExteriorDto>,
    pub intermediario_exterior: Option<ParteExteriorDto>,
    pub lote_id: Option<i64>,
    pub modelo: DocumentoModelo,
    pub serie: String,
    pub numero_documento: i64,
    pub chave_acesso: Option<String>,
    pub codigo_verificacao: Option<String>,
    pub ambiente: i32,
    pub tipo_emissao: i32,
    pub status: DocumentoStatus,
    pub processamento_mensagem: Option<String>,
    pub data_emissao: DateTime<Utc>,
    pub data_competencia: Option<NaiveDate>,
    pub serie_numero_automatico: bool,
    pub municipio_emissao_id: i64,
    pub municipio_prestacao_id: i64,
    pub protocolo_autorizacao: Option<String>,
    pub valor_total_servicos_produtos: Decimal,
    pub valor_liquido: Decimal,
    pub valor_recebido_intermediario: Option<Decimal>,
    pub desconto_incondicionado: Decimal,
    pub desconto_condicionado: Decimal,
    pub trib_issqn: TribIssqn,
    pub reg_esp_trib_issqn: RegEspTribIssqn,
    pub issqn_exigibilidade_suspensa: bool,
    pub issqn_retencao: bool,
    pub issqn_beneficio_municipal: bool,
    pub issqn_deducao_reducao_bc: bool,
    pub issqn_aliquota: Option<Decimal>,
    pub issqn_base_calculo: Option<Decimal>,
    pub issqn_valor: Option<Decimal>,
    pub cst_pis_cofins: CstPisCofins,
    pub tp_ret_pis_cofins_csll: TpRetPisCofinsCsll,
    pub valor_irrf: Option<Decimal>,
    pub valor_contrib_sociais_retidas: Option<Decimal>,
    pub valor_contrib_previdenciaria_retida: Option<Decimal>,
    pub numero_doc_responsabilidade_tecnica: Option<String>,
    pub documento_referencia: Option<String>,
    pub informacoes_complementares: Option<String>,
    pub numero_pedido_ordem_b2b: Option<String>,
}

#[derive(Serialize)]
pub struct DocumentoDto {
    #[serde(flatten)]
    pub list: DocumentoListDto,
    pub xml_assinado: Option<String>,
    pub xml_processado: Option<String>,
}

impl From<Documento> for DocumentoListDto {
    fn from(d: Documento) -> Self {
        Self {
            id: d.id,
            empresa_id: d.empresa_id,
            cliente_id: d.cliente_id,
            intermediario_id: d.intermediario_id,
            tomador_localidade: localidade_from_db(d.tomador_localidade)
                .expect("tomador_localidade inválida"),
            intermediario_localidade: localidade_from_db(d.intermediario_localidade)
                .expect("intermediario_localidade inválida"),
            tomador_exterior: decode_parte_exterior(&d.tomador_exterior),
            intermediario_exterior: decode_parte_exterior(&d.intermediario_exterior),
            lote_id: d.lote_id,
            modelo: documento_modelo_from_db(d.modelo).expect("modelo inválido"),
            serie: d.serie,
            numero_documento: d.numero_documento,
            chave_acesso: d.chave_acesso,
            codigo_verificacao: d.codigo_verificacao,
            ambiente: d.ambiente,
            tipo_emissao: d.tipo_emissao,
            status: documento_status_from_db(d.status).expect("status inválido"),
            processamento_mensagem: d.processamento_mensagem,
            data_emissao: d.data_emissao,
            data_competencia: d.data_competencia,
            serie_numero_automatico: d.serie_numero_automatico != 0,
            municipio_emissao_id: d.municipio_emissao_id,
            municipio_prestacao_id: d.municipio_prestacao_id,
            protocolo_autorizacao: d.protocolo_autorizacao,
            valor_total_servicos_produtos: d.valor_total_servicos_produtos,
            valor_liquido: d.valor_liquido,
            valor_recebido_intermediario: d.valor_recebido_intermediario,
            desconto_incondicionado: d.desconto_incondicionado,
            desconto_condicionado: d.desconto_condicionado,
            trib_issqn: trib_issqn_from_db(d.trib_issqn).expect("trib_issqn inválido"),
            reg_esp_trib_issqn: reg_esp_trib_from_db(d.reg_esp_trib_issqn)
                .expect("reg_esp_trib_issqn inválido"),
            issqn_exigibilidade_suspensa: d.issqn_exigibilidade_suspensa != 0,
            issqn_retencao: d.issqn_retencao != 0,
            issqn_beneficio_municipal: d.issqn_beneficio_municipal != 0,
            issqn_deducao_reducao_bc: d.issqn_deducao_reducao_bc != 0,
            issqn_aliquota: d.issqn_aliquota,
            issqn_base_calculo: d.issqn_base_calculo,
            issqn_valor: d.issqn_valor,
            cst_pis_cofins: cst_pis_cofins_from_db(d.cst_pis_cofins)
                .expect("cst_pis_cofins inválido"),
            tp_ret_pis_cofins_csll: tp_ret_from_db(d.tp_ret_pis_cofins_csll)
                .expect("tp_ret_pis_cofins_csll inválido"),
            valor_irrf: d.valor_irrf,
            valor_contrib_sociais_retidas: d.valor_contrib_sociais_retidas,
            valor_contrib_previdenciaria_retida: d.valor_contrib_previdenciaria_retida,
            numero_doc_responsabilidade_tecnica: d.numero_doc_responsabilidade_tecnica,
            documento_referencia: d.documento_referencia,
            informacoes_complementares: d.informacoes_complementares,
            numero_pedido_ordem_b2b: d.numero_pedido_ordem_b2b,
        }
    }
}

impl From<Documento> for DocumentoDto {
    fn from(d: Documento) -> Self {
        Self {
            list: DocumentoListDto::from(d.clone()),
            xml_assinado: d.xml_assinado,
            xml_processado: d.xml_processado,
        }
    }
}

#[derive(Serialize)]
pub struct ItemDto {
    pub id: i64,
    pub documento_fiscal_id: i64,
    pub numero_item: i32,
    pub descricao: String,
    pub descricao_servico: Option<String>,
    pub codigo_ncm: Option<String>,
    pub codigo_tributacao_nacional: Option<String>,
    pub codigo_tributacao_municipal: Option<String>,
    pub cna_id: Option<i64>,
    pub conta_fiscal_id: Option<i64>,
    pub quantidade: Decimal,
    pub valor_unitario: Decimal,
    pub valor_total_bruto: Decimal,
    pub valor_desconto: Option<Decimal>,
}

impl From<Item> for ItemDto {
    fn from(i: Item) -> Self {
        Self {
            id: i.id,
            documento_fiscal_id: i.documento_fiscal_id,
            numero_item: i.numero_item,
            descricao: i.descricao,
            descricao_servico: i.descricao_servico,
            codigo_ncm: i.codigo_ncm,
            codigo_tributacao_nacional: i.codigo_tributacao_nacional,
            codigo_tributacao_municipal: i.codigo_tributacao_municipal,
            cna_id: i.cna_id,
            conta_fiscal_id: i.conta_fiscal_id,
            quantidade: i.quantidade,
            valor_unitario: i.valor_unitario,
            valor_total_bruto: i.valor_total_bruto,
            valor_desconto: i.valor_desconto,
        }
    }
}

#[derive(Serialize)]
pub struct TributoDto {
    pub id: i64,
    pub item_id: i64,
    pub tipo_imposto: TipoImposto,
    pub cst_csosn: Option<String>,
    pub base_calculo: Option<Decimal>,
    pub aliquota: Option<Decimal>,
    pub valor_imposto: Option<Decimal>,
    pub iss_retido: Option<i8>,
    pub classificacao_trib_ibscbs: Option<String>,
    pub aliquota_ibs: Option<Decimal>,
    pub valor_ibs: Option<Decimal>,
    pub aliquota_cbs: Option<Decimal>,
    pub valor_cbs: Option<Decimal>,
}

impl From<Tributo> for TributoDto {
    fn from(t: Tributo) -> Self {
        Self {
            id: t.id,
            item_id: t.item_id,
            tipo_imposto: tipo_imposto_from_db(t.tipo_imposto).expect("tipo_imposto inválido"),
            cst_csosn: t.cst_csosn,
            base_calculo: t.base_calculo,
            aliquota: t.aliquota,
            valor_imposto: t.valor_imposto,
            iss_retido: t.iss_retido,
            classificacao_trib_ibscbs: t.classificacao_trib_ibscbs,
            aliquota_ibs: t.aliquota_ibs,
            valor_ibs: t.valor_ibs,
            aliquota_cbs: t.aliquota_cbs,
            valor_cbs: t.valor_cbs,
        }
    }
}

#[derive(Serialize)]
pub struct EventoDto {
    pub id: i64,
    pub documento_fiscal_id: i64,
    pub tipo_evento: TipoEvento,
    pub sequencial_evento: i32,
    pub protocolo_evento: Option<String>,
    pub descricao_motivo: Option<String>,
    pub data_hora_evento: DateTime<Utc>,
    pub ambiente: i32,
    pub xml_evento: Option<String>,
}

impl From<Evento> for EventoDto {
    fn from(e: Evento) -> Self {
        Self {
            id: e.id,
            documento_fiscal_id: e.documento_fiscal_id,
            tipo_evento: tipo_evento_from_db(e.tipo_evento).expect("tipo_evento inválido"),
            sequencial_evento: e.sequencial_evento,
            protocolo_evento: e.protocolo_evento,
            descricao_motivo: e.descricao_motivo,
            data_hora_evento: e.data_hora_evento,
            ambiente: e.ambiente,
            xml_evento: e.xml_evento,
        }
    }
}

#[derive(Serialize)]
pub struct LogRetornoDto {
    pub id: i64,
    pub documento_fiscal_id: Option<i64>,
    pub lote_id: Option<i64>,
    pub codigo_erro: String,
    pub mensagem_retorno: String,
    pub correcao_sugerida: Option<String>,
    pub data_registro: DateTime<Utc>,
}

impl From<LogRetorno> for LogRetornoDto {
    fn from(l: LogRetorno) -> Self {
        Self {
            id: l.id,
            documento_fiscal_id: l.documento_fiscal_id,
            lote_id: l.lote_id,
            codigo_erro: l.codigo_erro,
            mensagem_retorno: l.mensagem_retorno,
            correcao_sugerida: l.correcao_sugerida,
            data_registro: l.data_registro,
        }
    }
}

pub fn encode_certificado_senha(plain: &str) -> String {
    format!("enc:v1:{plain}")
}

pub fn decode_certificado_senha(stored: &str) -> String {
    stored
        .strip_prefix("enc:v1:")
        .unwrap_or(stored)
        .to_string()
}

#[derive(Serialize)]
pub struct CncMensagemPublic {
    pub codigo: Option<String>,
    pub descricao: Option<String>,
}

#[derive(Serialize)]
pub struct CncCadastroItemPublic {
    pub inscricao: Option<String>,
    pub tp_inscricao: Option<String>,
    pub razao_social: Option<String>,
    pub inscricao_municipal: Option<String>,
    pub email: Option<String>,
    pub telefone: Option<String>,
    pub codigo_municipio_ibge: Option<String>,
    pub municipio_id: Option<i64>,
    pub cep: Option<String>,
    pub logradouro: Option<String>,
    pub numero: Option<String>,
    pub complemento: Option<String>,
    pub bairro: Option<String>,
    /// Valor `opSimpNac` (1–3) retornado pelo CNC, quando informado.
    pub op_simp_nac: Option<i32>,
    pub regime_tributario_sugerido: Option<RegimeTributario>,
}

#[derive(Serialize)]
pub struct CncConsultaCadPublic {
    pub status_processamento: Option<String>,
    pub tipo_ambiente: Option<String>,
    pub data_hora_processamento: Option<String>,
    pub alertas: Vec<CncMensagemPublic>,
    pub erros: Vec<CncMensagemPublic>,
    pub itens: Vec<CncCadastroItemPublic>,
}

#[derive(Serialize)]
pub struct TransmitirDocumentoResponse {
    pub sucesso: bool,
    pub http_status: u16,
    pub id_dps: Option<String>,
    pub chave_acesso: Option<String>,
    pub nfse_xml: Option<String>,
    pub alertas: Vec<super::nfse::MensagemProcessamento>,
    pub erros: Vec<super::nfse::MensagemProcessamento>,
    pub documento: DocumentoDto,
}
