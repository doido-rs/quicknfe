use chrono::{DateTime, FixedOffset, Utc};
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
    /// Proxy local: município de incidência com parametrização ADN sincronizada (convênio ativo).
    pub municipio_incidencia_sn_ativo: bool,
}

pub async fn load_dps_context(
    db: &doido::model::sea_orm::DatabaseConnection,
    doc: Documento,
) -> Result<DpsContext, ApiError> {
    let itens = ItemEntity::find()
        .filter(ItemColumn::DocumentoFiscalId.eq(doc.id))
        .order_by_asc(ItemColumn::NumeroItem)
        .all(db)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    load_dps_context_with_itens(db, doc, itens).await
}

pub async fn load_dps_context_with_itens(
    db: &doido::model::sea_orm::DatabaseConnection,
    doc: Documento,
    itens: Vec<ItemDoc>,
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

    let tomador_exterior = decode_exterior(&doc.tomador_exterior);
    let intermediario_exterior = decode_exterior(&doc.intermediario_exterior);

    let mut ctx = DpsContext {
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
        municipio_incidencia_sn_ativo: false,
    };
    ctx.municipio_incidencia_sn_ativo =
        super::catalog_item_incidencia::municipio_incidencia_sn_ativo(db, &ctx)
            .await
            .unwrap_or(false);
    Ok(ctx)
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

/// `TSDateTimeUTC`: sem fração de segundos; offset `-03:00` (horário de Brasília).
pub fn format_dh_emi_dps(dt: DateTime<Utc>) -> String {
    let brt = FixedOffset::west_opt(3 * 3600).expect("BRT offset");
    dt.with_timezone(&brt)
        .format("%Y-%m-%dT%H:%M:%S-03:00")
        .to_string()
}

pub fn only_digits(s: &str) -> String {
    s.chars().filter(|c| c.is_ascii_digit()).collect()
}

/// `TSTelefone` (6–20 dígitos). Retorna `None` se inválido — tag `fone` é 0-1 e não pode ir vazia.
pub fn telefone_para_dps(raw: Option<&str>) -> Option<String> {
    let raw = raw?.trim();
    if raw.is_empty() {
        return None;
    }
    let digits = only_digits(raw);
    if (6..=20).contains(&digits.len()) {
        Some(digits)
    } else {
        None
    }
}

pub fn cpf_cnpj_tag(cpf_cnpj: &str) -> (&'static str, String) {
    let digits = only_digits(cpf_cnpj);
    if digits.len() <= 11 {
        ("CPF", format!("{:0>11}", digits))
    } else {
        ("CNPJ", format!("{:0>14}", digits))
    }
}

fn pad_ibge7(code: &str) -> String {
    let digits = only_digits(code);
    format!("{:0>7}", digits)
}

/// Valor de `<serie>` no XML (numérico, faixa emissor próprio quando `AUTO`).
pub fn serie_dps_para_xml(serie: &str) -> String {
    let digits = only_digits(serie);
    if digits.is_empty() {
        return "1".into();
    }
    format!("{}", digits.parse::<u64>().unwrap_or(1).min(99_999))
}

/// Série com 5 dígitos no atributo `Id` (TSIdDPS), alinhada a [`serie_dps_para_xml`].
fn serie_id_5(serie: &str) -> String {
    format!("{:05}", serie_dps_para_xml(serie).parse::<u64>().unwrap_or(1))
}

/// Número DPS com 15 dígitos no atributo `Id` (TSIdDPS).
fn ndps_id_15(numero: i64) -> Result<String, ApiError> {
    if numero <= 0 {
        return Err(ApiError::Validation(
            "número da DPS deve ser positivo para gerar o identificador infDPS".into(),
        ));
    }
    if numero > 999_999_999_999_999 {
        return Err(ApiError::Validation(
            "número da DPS excede 15 dígitos no identificador infDPS".into(),
        ));
    }
    Ok(format!("{:015}", numero))
}

/// Identificador `infDPS/@Id` — literal `DPS` + 42 dígitos (RN E0004 / TSIdDPS).
///
/// Composição: Cód.Mun.Emi. (7) + Tipo Inscrição Federal (1) + Inscrição Federal (14) +
/// Série DPS (5) + Núm. DPS (15).
pub fn gerar_id_inf_dps(ctx: &DpsContext) -> Result<String, ApiError> {
    let doc = &ctx.doc;
    let c_loc_emi = pad_ibge7(&ctx.municipio_emissao.codigo_ibge);
    if c_loc_emi.len() != 7 {
        return Err(ApiError::Validation(
            "código IBGE do município de emissão inválido para identificador da DPS".into(),
        ));
    }

    let insc_digits = only_digits(&ctx.prestador.cpf_cnpj);
    if insc_digits.is_empty() {
        return Err(ApiError::Validation(
            "CPF/CNPJ do prestador é obrigatório para identificador da DPS".into(),
        ));
    }
    let tp_insc = if insc_digits.len() <= 11 { "1" } else { "2" };
    let insc_14 = format!("{:0>14}", insc_digits);
    if insc_14.len() != 14 {
        return Err(ApiError::Validation(
            "inscrição federal do prestador inválida para identificador da DPS".into(),
        ));
    }

    let serie_5 = serie_id_5(&doc.serie);
    let ndps_15 = ndps_id_15(doc.numero_documento)?;

    Ok(format!(
        "DPS{c_loc_emi}{tp_insc}{insc_14}{serie_5}{ndps_15}"
    ))
}

#[cfg(test)]
mod tests {
    use super::telefone_para_dps;

    #[test]
    fn format_dh_emi_dps_sem_fracoes() {
        let dt = chrono::DateTime::parse_from_rfc3339("2026-10-21T15:30:45Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        assert_eq!(
            super::format_dh_emi_dps(dt),
            "2026-10-21T12:30:45-03:00"
        );
    }

    #[test]
    fn telefone_para_dps_aceita_ddd_mais_numero() {
        assert_eq!(
            telefone_para_dps(Some("(21) 98765-4321")).as_deref(),
            Some("21987654321")
        );
    }

    #[test]
    fn telefone_para_dps_omite_vazio_ou_so_mascara() {
        assert_eq!(telefone_para_dps(Some("")), None);
        assert_eq!(telefone_para_dps(Some("   ")), None);
        assert_eq!(telefone_para_dps(Some("( ) ----")), None);
        assert_eq!(telefone_para_dps(None), None);
    }

    #[test]
    fn telefone_para_dps_rejeita_poucos_digitos() {
        assert_eq!(telefone_para_dps(Some("12345")), None);
    }
}
