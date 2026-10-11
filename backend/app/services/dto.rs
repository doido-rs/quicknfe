use chrono::{DateTime, NaiveDate, Utc};
use doido::model::sea_orm::prelude::Decimal;
use serde::Serialize;

use crate::models::certificado_digital::Model as Certificado;
use crate::models::cliente::Model as Cliente;
use crate::models::documento_fiscal::Model as Documento;
use crate::models::company::Model as Company;
use crate::models::empresa::Model as Empresa;
use crate::models::regime_tributario::RegimeTributario;
use crate::models::evento_documento_fiscal::Model as Evento;
use crate::models::intermediario::Model as Intermediario;
use crate::models::item_documento_fiscal::Model as Item;
use crate::models::log_mensagem_retorno::Model as LogRetorno;
use crate::models::lote_transmissao::Model as Lote;
use crate::models::municipio::Model as Municipio;
use crate::models::tributo_item::Model as Tributo;
use crate::models::user::Model as User;

#[derive(Serialize)]
pub struct UserPublic {
    pub id: i64,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<User> for UserPublic {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            email: u.email,
            created_at: u.created_at,
            updated_at: u.updated_at,
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
        Ok(Self {
            id: e.id,
            cpf_cnpj: e.cpf_cnpj,
            razao_social: e.razao_social,
            inscricao_municipal: e.inscricao_municipal,
            inscricao_estadual: e.inscricao_estadual,
            regime_tributario: RegimeTributario::parse(&e.regime_tributario).map_err(|_| {
                crate::services::api_error::ApiError::Internal(format!(
                    "regime_tributario inválido no banco para empresa {}",
                    e.id
                ))
            })?,
            municipio_id: e.municipio_id,
            municipio: MunicipioDto::from(m),
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
    pub papel: String,
}

#[derive(Serialize)]
pub struct MeResponse {
    pub user: UserPublic,
    pub companies: Vec<CompanyMembership>,
    pub company_ativa: Option<CompanyPublic>,
    pub prestadores: Vec<EmpresaPublic>,
    pub prestador_ativo: Option<EmpresaPublic>,
    pub papel: Option<String>,
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
        }
    }
}

#[derive(Serialize)]
pub struct CertificadoDto {
    pub id: i64,
    pub empresa_id: i64,
    pub tipo: String,
    pub caminho_arquivo: Option<String>,
    pub data_validade: DateTime<Utc>,
    pub criado_em: DateTime<Utc>,
}

impl From<Certificado> for CertificadoDto {
    fn from(c: Certificado) -> Self {
        Self {
            id: c.id,
            empresa_id: c.empresa_id,
            tipo: c.tipo,
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
    pub status_lote: String,
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
            status_lote: l.status_lote,
            data_recebimento: l.data_recebimento,
        }
    }
}

#[derive(Serialize)]
pub struct DocumentoListDto {
    pub id: i64,
    pub empresa_id: i64,
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
            lote_id: d.lote_id,
            modelo: d.modelo,
            serie: d.serie,
            numero_documento: d.numero_documento,
            chave_acesso: d.chave_acesso,
            codigo_verificacao: d.codigo_verificacao,
            ambiente: d.ambiente,
            tipo_emissao: d.tipo_emissao,
            status: d.status,
            data_emissao: d.data_emissao,
            data_competencia: d.data_competencia,
            codigo_municipio_emissao: d.codigo_municipio_emissao,
            protocolo_autorizacao: d.protocolo_autorizacao,
            valor_total_servicos_produtos: d.valor_total_servicos_produtos,
            valor_liquido: d.valor_liquido,
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
    pub codigo_ncm: Option<String>,
    pub codigo_tributacao_nacional: Option<String>,
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
            codigo_ncm: i.codigo_ncm,
            codigo_tributacao_nacional: i.codigo_tributacao_nacional,
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
    pub tipo_imposto: String,
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
            tipo_imposto: t.tipo_imposto,
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
    pub tipo_evento: String,
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
            tipo_evento: e.tipo_evento,
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
