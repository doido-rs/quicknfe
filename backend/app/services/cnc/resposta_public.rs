use doido::model::sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

use super::tipos::{CadastroMunicipalCnc, MensagemCnc, RespostaConsultaCad};
use crate::models::empresa::Entity as EmpresaEntity;
use crate::models::municipio::{Column as MunicipioColumn, Entity as MunicipioEntity};
use crate::models::regime_tributario::RegimeTributario;
use crate::services::api_error::ApiError;
use crate::services::dto::{CncCadastroItemPublic, CncConsultaCadPublic, CncMensagemPublic};
use crate::services::nfse::op_simp_nac::{
    op_simp_nac_from_cnc_json, regime_sugerido_para_op_simp_nac,
};

pub async fn resposta_publica(
    db: &impl doido::model::sea_orm::ConnectionTrait,
    raw: RespostaConsultaCad,
) -> Result<CncConsultaCadPublic, ApiError> {
    let mut itens = Vec::new();
    for entry in raw.lista_cadastro_municipal {
        let (op_simp_nac, regime_tributario_sugerido) = opcao_simples_do_cadastro(&entry);
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
            op_simp_nac,
            regime_tributario_sugerido,
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

fn opcao_simples_do_cadastro(
    entry: &CadastroMunicipalCnc,
) -> (Option<i32>, Option<RegimeTributario>) {
    let Some(ref raw) = entry.opcao_simples_nacional else {
        return (None, None);
    };
    let Some(op) = op_simp_nac_from_cnc_json(raw) else {
        return (None, None);
    };
    (
        Some(op.as_i32()),
        Some(regime_sugerido_para_op_simp_nac(op)),
    )
}

pub fn op_simp_nac_de_resposta(raw: &RespostaConsultaCad) -> Option<i32> {
    for entry in &raw.lista_cadastro_municipal {
        let (op, _) = opcao_simples_do_cadastro(entry);
        if op.is_some() {
            return op;
        }
    }
    None
}

/// Persiste snapshot CNC para validação E0160 antes de assinar/transmitir.
pub async fn gravar_op_simp_nac_verificado(
    db: &impl doido::model::sea_orm::ConnectionTrait,
    empresa_id: i64,
    raw: &RespostaConsultaCad,
) -> Result<(), ApiError> {
    let Some(op) = op_simp_nac_de_resposta(raw) else {
        return Ok(());
    };
    let mut record: crate::models::empresa::ActiveModel = EmpresaEntity::find_by_id(empresa_id)
        .one(db)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::Validation("prestador não encontrado".into()))?
        .into();
    record.op_simp_nac_verificado = Set(Some(op));
    record
        .update(db)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(())
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
