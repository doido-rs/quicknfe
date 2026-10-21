//! Campos derivados para listagem e detalhe de notas fiscais na API.

use std::collections::HashMap;

use doido::model::sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Serialize;

use crate::models::cliente::{Column as ClienteColumn, Entity as ClienteEntity};
use crate::models::documento_fiscal::Model as Documento;
use crate::models::empresa::{Column as EmpresaColumn, Entity as EmpresaEntity};
use crate::models::intermediario::{Column as IntermediarioColumn, Entity as IntermediarioEntity};
use crate::services::api_error::ApiError;
use crate::services::dto::{DocumentoDto, DocumentoListDto, MunicipioDto, ParteExteriorDto};

#[derive(Serialize)]
pub struct DocumentoListItemDto {
    #[serde(flatten)]
    pub inner: DocumentoListDto,
    pub prestador_razao_social: String,
    pub tomador_razao_social: Option<String>,
    pub intermediario_razao_social: Option<String>,
}

#[derive(Serialize)]
pub struct DocumentoDetalheDto {
    #[serde(flatten)]
    pub documento: DocumentoDto,
    pub prestador_razao_social: String,
    pub tomador_razao_social: Option<String>,
    pub intermediario_razao_social: Option<String>,
    pub municipio_emissao: MunicipioDto,
    pub municipio_prestacao: MunicipioDto,
}

struct PartesNomes {
    prestadores: HashMap<i64, String>,
    clientes: HashMap<i64, String>,
    intermediarios: HashMap<i64, String>,
}

async fn carregar_nomes_partes(
    db: &impl doido::model::sea_orm::ConnectionTrait,
    docs: &[Documento],
) -> Result<PartesNomes, ApiError> {
    let empresa_ids: Vec<i64> = docs.iter().map(|d| d.empresa_id).collect();
    let cliente_ids: Vec<i64> = docs.iter().filter_map(|d| d.cliente_id).collect();
    let intermediario_ids: Vec<i64> = docs.iter().filter_map(|d| d.intermediario_id).collect();

    let prestadores = if empresa_ids.is_empty() {
        HashMap::new()
    } else {
        EmpresaEntity::find()
            .filter(EmpresaColumn::Id.is_in(empresa_ids))
            .all(db)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?
            .into_iter()
            .map(|e| (e.id, e.razao_social))
            .collect()
    };

    let clientes = if cliente_ids.is_empty() {
        HashMap::new()
    } else {
        ClienteEntity::find()
            .filter(ClienteColumn::Id.is_in(cliente_ids))
            .all(db)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?
            .into_iter()
            .map(|c| (c.id, c.razao_social))
            .collect()
    };

    let intermediarios = if intermediario_ids.is_empty() {
        HashMap::new()
    } else {
        IntermediarioEntity::find()
            .filter(IntermediarioColumn::Id.is_in(intermediario_ids))
            .all(db)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?
            .into_iter()
            .map(|i| (i.id, i.razao_social))
            .collect()
    };

    Ok(PartesNomes {
        prestadores,
        clientes,
        intermediarios,
    })
}

fn tomador_nome(doc: &Documento, partes: &PartesNomes) -> Option<String> {
    use crate::models::localidade_parte::LocalidadeParte;
    use crate::services::entity_enums::localidade_from_db;
    let loc = localidade_from_db(doc.tomador_localidade).ok()?;
    if loc == LocalidadeParte::Exterior {
        return decode_parte_exterior(&doc.tomador_exterior).map(|p| p.razao_social);
    }
    if loc == LocalidadeParte::NaoInformado {
        return None;
    }
    doc.cliente_id
        .and_then(|id| partes.clientes.get(&id).cloned())
}

fn intermediario_nome(doc: &Documento, partes: &PartesNomes) -> Option<String> {
    use crate::models::localidade_parte::LocalidadeParte;
    use crate::services::entity_enums::localidade_from_db;
    let loc = localidade_from_db(doc.intermediario_localidade).ok()?;
    if loc == LocalidadeParte::Exterior {
        return decode_parte_exterior(&doc.intermediario_exterior).map(|p| p.razao_social);
    }
    if loc == LocalidadeParte::NaoInformado {
        return None;
    }
    doc.intermediario_id
        .and_then(|id| partes.intermediarios.get(&id).cloned())
}

fn decode_parte_exterior(raw: &Option<String>) -> Option<ParteExteriorDto> {
    raw.as_ref()
        .and_then(|s| serde_json::from_str::<ParteExteriorDto>(s).ok())
}

pub async fn documentos_list_items(
    db: &impl doido::model::sea_orm::ConnectionTrait,
    docs: Vec<Documento>,
) -> Result<Vec<DocumentoListItemDto>, ApiError> {
    let partes = carregar_nomes_partes(db, &docs).await?;
    Ok(docs
        .into_iter()
        .map(|d| {
            let prestador_razao_social = partes
                .prestadores
                .get(&d.empresa_id)
                .cloned()
                .unwrap_or_else(|| format!("Prestador #{}", d.empresa_id));
            DocumentoListItemDto {
                inner: DocumentoListDto::from(d.clone()),
                prestador_razao_social,
                tomador_razao_social: tomador_nome(&d, &partes),
                intermediario_razao_social: intermediario_nome(&d, &partes),
            }
        })
        .collect())
}

pub async fn documento_detalhe(
    db: &impl doido::model::sea_orm::ConnectionTrait,
    doc: Documento,
) -> Result<DocumentoDetalheDto, ApiError> {
    let partes = carregar_nomes_partes(db, std::slice::from_ref(&doc)).await?;
    let municipio_emissao =
        crate::services::municipio::require_municipio(db, doc.municipio_emissao_id).await?;
    let municipio_prestacao =
        crate::services::municipio::require_municipio(db, doc.municipio_prestacao_id).await?;

    let prestador_razao_social = partes
        .prestadores
        .get(&doc.empresa_id)
        .cloned()
        .unwrap_or_else(|| format!("Prestador #{}", doc.empresa_id));

    Ok(DocumentoDetalheDto {
        documento: DocumentoDto::from(doc.clone()),
        prestador_razao_social,
        tomador_razao_social: tomador_nome(&doc, &partes),
        intermediario_razao_social: intermediario_nome(&doc, &partes),
        municipio_emissao: MunicipioDto::from(&municipio_emissao),
        municipio_prestacao: MunicipioDto::from(&municipio_prestacao),
    })
}
