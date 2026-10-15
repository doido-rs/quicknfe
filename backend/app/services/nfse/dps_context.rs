use doido::model::sea_orm::prelude::Decimal;
use doido::model::sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde::Deserialize;

use crate::models::cliente::Entity as ClienteEntity;
use crate::models::documento_fiscal::Model as Documento;
use crate::models::empresa::Entity as EmpresaEntity;
use crate::models::intermediario::Entity as IntermediarioEntity;
use crate::models::item_documento_fiscal::Entity as ItemEntity;
use crate::models::item_documento_fiscal::Column as ItemColumn;
use crate::models::municipio::Entity as MunicipioEntity;
use crate::models::municipio::Model as Municipio;
use crate::models::empresa::Model as Empresa;
use crate::models::cliente::Model as Cliente;
use crate::models::intermediario::Model as Intermediario;
use crate::models::item_documento_fiscal::Model as ItemDoc;
use crate::services::api_error::ApiError;

#[derive(Debug, Clone, Deserialize)]
pub struct ParteExterior {
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

#[derive(Debug, Clone)]
pub struct DpsContext {
    pub doc: Documento,
    pub prestador: Empresa,
    pub municipio_emissao: Municipio,
    pub municipio_prestacao: Municipio,
    pub municipio_prestador: Municipio,
    pub tomador: Option<Cliente>,
    pub tomador_municipio: Option<Municipio>,
    pub tomador_exterior: Option<ParteExterior>,
    pub intermediario: Option<Intermediario>,
    pub intermediario_municipio: Option<Municipio>,
    pub intermediario_exterior: Option<ParteExterior>,
    pub itens: Vec<ItemDoc>,
}

pub async fn load_dps_context(
    db: &doido::model::sea_orm::DatabaseConnection,
    doc: Documento,
) -> Result<DpsContext, ApiError> {
    let prestador = EmpresaEntity::find_by_id(doc.empresa_id)
        .one(db)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::Validation("prestador não encontrado".into()))?;

    let municipio_emissao = load_municipio(db, doc.municipio_emissao_id).await?;
    let municipio_prestacao = load_municipio(db, doc.municipio_prestacao_id).await?;
    let municipio_prestador = load_municipio(db, prestador.municipio_id).await?;

    let tomador = match doc.cliente_id {
        Some(id) => ClienteEntity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?,
        None => None,
    };
    let tomador_municipio = if let Some(ref c) = tomador {
        match c.municipio_id {
            Some(mid) => Some(load_municipio(db, mid).await?),
            None => None,
        }
    } else {
        None
    };

    let intermediario = match doc.intermediario_id {
        Some(id) => IntermediarioEntity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?,
        None => None,
    };
    let intermediario_municipio = if let Some(ref i) = intermediario {
        match i.municipio_id {
            Some(mid) => Some(load_municipio(db, mid).await?),
            None => None,
        }
    } else {
        None
    };

    let itens = ItemEntity::find()
        .filter(ItemColumn::DocumentoFiscalId.eq(doc.id))
        .order_by_asc(ItemColumn::NumeroItem)
        .all(db)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let tomador_exterior = decode_exterior(&doc.tomador_exterior);
    let intermediario_exterior = decode_exterior(&doc.intermediario_exterior);

    Ok(DpsContext {
        doc,
        prestador,
        municipio_emissao,
        municipio_prestacao,
        municipio_prestador,
        tomador,
        tomador_municipio,
        tomador_exterior,
        intermediario,
        intermediario_municipio,
        intermediario_exterior,
        itens,
    })
}

async fn load_municipio(
    db: &doido::model::sea_orm::DatabaseConnection,
    id: i64,
) -> Result<Municipio, ApiError> {
    MunicipioEntity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::Validation(format!("município id {id} não encontrado")))
}

fn decode_exterior(raw: &Option<String>) -> Option<ParteExterior> {
    raw.as_ref()
        .filter(|s| !s.trim().is_empty())
        .and_then(|s| serde_json::from_str(s).ok())
}

pub fn decimal_xml(d: &Decimal) -> String {
    d.normalize().to_string()
}

pub fn only_digits(s: &str) -> String {
    s.chars().filter(|c| c.is_ascii_digit()).collect()
}

pub fn cpf_cnpj_tag(cpf_cnpj: &str) -> (&'static str, String) {
    let digits = only_digits(cpf_cnpj);
    if digits.len() <= 11 {
        ("CPF", format!("{:0>11}", digits))
    } else {
        ("CNPJ", format!("{:0>14}", digits))
    }
}

pub fn gerar_id_inf_dps(doc_id: i64) -> String {
    format!("DPS{:042}", doc_id)
}
