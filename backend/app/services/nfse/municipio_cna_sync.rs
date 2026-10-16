//! Sincroniza parametrização municipal (ADN) → `municipio_cna_parametrizacao`.

use chrono::Utc;
use doido::model::sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter,
    Set, TransactionTrait,
};

use crate::connectors::adn;
use crate::models::cna::{Column as CnaColumn, Entity as CnaEntity};
use crate::models::municipio::{Column as MunicipioColumn, Entity as MunicipioEntity};
use crate::models::municipio_cna_parametrizacao::{
    ActiveModel as ParamActive, Column as ParamColumn, Entity as ParamEntity,
};

#[derive(Debug, Clone, Default)]
pub struct MunicipioSyncResult {
    pub ibge: String,
    pub rows: u32,
    pub error: Option<String>,
}

pub async fn sync_municipio_by_ibge(
    db: &DatabaseConnection,
    codigo_ibge: &str,
) -> doido_core::Result<MunicipioSyncResult> {
    let mut result = MunicipioSyncResult {
        ibge: codigo_ibge.to_string(),
        ..Default::default()
    };

    let municipio = match MunicipioEntity::find()
        .filter(MunicipioColumn::CodigoIbge.eq(codigo_ibge.trim()))
        .one(db)
        .await
    {
        Ok(Some(m)) => m,
        Ok(None) => {
            result.error = Some("município não cadastrado".into());
            return Ok(result);
        }
        Err(e) => return Err(doido_core::anyhow::anyhow!("query municipio: {e}").into()),
    };

    let codigos_nacionais: Vec<String> = CnaEntity::find()
        .all(db)
        .await
        .map_err(|e| doido_core::anyhow::anyhow!("list cnas: {e}"))?
        .into_iter()
        .map(|c| c.codigo)
        .collect();

    let params = match adn::get_aliquotas_municipio(&municipio.codigo_ibge, &codigos_nacionais).await
    {
        Ok(p) => p,
        Err(e) => {
            result.error = Some(e.to_string());
            return Ok(result);
        }
    };

    let txn = db
        .begin()
        .await
        .map_err(|e| doido_core::anyhow::anyhow!("begin txn: {e}"))?;

    ParamEntity::delete_many()
        .filter(ParamColumn::MunicipioId.eq(municipio.id))
        .exec(&txn)
        .await
        .map_err(|e| doido_core::anyhow::anyhow!("delete parametrizacao: {e}"))?;

    let synced_at = Utc::now().naive_utc();
    for p in &params {
        let cna_id = CnaEntity::find()
            .filter(CnaColumn::Codigo.eq(&p.codigo_trib_nac))
            .one(&txn)
            .await
            .map_err(|e| doido_core::anyhow::anyhow!("query cna: {e}"))?
            .map(|c| c.id);

        ParamActive {
            municipio_id: Set(municipio.id),
            cna_id: Set(cna_id),
            codigo_trib_nac: Set(p.codigo_trib_nac.clone()),
            codigo_trib_mun: Set(p.codigo_trib_mun.clone()),
            codigo_parametrizacao: Set(p.codigo_parametrizacao.clone()),
            aliquota: Set(p.aliquota),
            vigencia_inicio: Set(p.vigencia_inicio),
            vigencia_fim: Set(p.vigencia_fim),
            synced_at: Set(synced_at),
            ..Default::default()
        }
        .insert(&txn)
        .await
        .map_err(|e| doido_core::anyhow::anyhow!("insert parametrizacao: {e}"))?;
        result.rows += 1;
    }

    txn.commit()
        .await
        .map_err(|e| doido_core::anyhow::anyhow!("commit txn: {e}"))?;

    Ok(result)
}

pub async fn sync_municipios_batch(
    db: &DatabaseConnection,
    ibge_codes: &[String],
    sleep_ms: u64,
) -> Vec<MunicipioSyncResult> {
    let mut results = Vec::with_capacity(ibge_codes.len());
    for ibge in ibge_codes {
        let r = sync_municipio_by_ibge(db, ibge)
            .await
            .unwrap_or_else(|e| MunicipioSyncResult {
                ibge: ibge.clone(),
                error: Some(e.to_string()),
                ..Default::default()
            });
        results.push(r);
        if sleep_ms > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(sleep_ms)).await;
        }
    }
    results
}

pub async fn municipio_tem_parametrizacao(
    db: &impl ConnectionTrait,
    municipio_id: i64,
) -> doido_core::Result<bool> {
    use doido::model::sea_orm::PaginatorTrait;
    let n = ParamEntity::find()
        .filter(ParamColumn::MunicipioId.eq(municipio_id))
        .count(db)
        .await
        .map_err(|e| doido_core::anyhow::anyhow!("count parametrizacao: {e}"))?;
    Ok(n > 0)
}
