use doido::model::sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use super::tipos::{MensagemCnc, RespostaConsultaCad};
use crate::models::municipio::{Column as MunicipioColumn, Entity as MunicipioEntity};
use crate::services::api_error::ApiError;
use crate::services::dto::{CncCadastroItemPublic, CncConsultaCadPublic, CncMensagemPublic};

pub async fn resposta_publica(
    db: &impl doido::model::sea_orm::ConnectionTrait,
    raw: RespostaConsultaCad,
) -> Result<CncConsultaCadPublic, ApiError> {
    let mut itens = Vec::new();
    for entry in raw.lista_cadastro_municipal {
        let Some(inf) = entry.inf_cad else { continue };
        let codigo_ibge = entry.codigo_municipio.map(|c| c.to_string());
        let municipio_id = if let Some(ref cod) = codigo_ibge {
            municipio_id_por_ibge(db, cod).await?
        } else {
            None
        };
        let end = inf.info_endereco.as_ref();
        itens.push(CncCadastroItemPublic {
            inscricao: inf.inscricao.clone(),
            tp_inscricao: inf.tp_inscricao.clone(),
            razao_social: inf.razao_social.clone(),
            inscricao_municipal: inf.inscricao_municipal.clone(),
            email: inf.email.clone(),
            telefone: inf.telefone.clone(),
            codigo_municipio_ibge: codigo_ibge,
            municipio_id,
            cep: end.and_then(|e| e.cep.clone()),
            logradouro: end.and_then(|e| e.logradouro.clone()),
            numero: end.and_then(|e| e.numero.clone()),
            complemento: end.and_then(|e| e.complemento.clone()),
            bairro: end.and_then(|e| e.bairro.clone()),
        });
    }

    Ok(CncConsultaCadPublic {
        status_processamento: raw.status_processamento,
        tipo_ambiente: raw.tipo_ambiente,
        data_hora_processamento: raw.data_hora_processamento,
        alertas: map_msgs(raw.alertas),
        erros: map_msgs(raw.erros),
        itens,
    })
}

fn map_msgs(msgs: Vec<MensagemCnc>) -> Vec<CncMensagemPublic> {
    msgs.into_iter()
        .map(|m| CncMensagemPublic {
            codigo: m.codigo,
            descricao: m.descricao,
        })
        .collect()
}

async fn municipio_id_por_ibge(
    db: &impl doido::model::sea_orm::ConnectionTrait,
    codigo_ibge: &str,
) -> Result<Option<i64>, ApiError> {
    let row = MunicipioEntity::find()
        .filter(MunicipioColumn::CodigoIbge.eq(codigo_ibge))
        .one(db)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(row.map(|m| m.id))
}
