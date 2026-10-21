//! Orquestração da carga diária (Anexo A, lista nacional, coordenador municipal).

use doido::model::sea_orm::{EntityTrait, QuerySelect};
use serde_json::json;

use crate::models::catalog_sync_kind::CatalogSyncKind;
use crate::models::catalog_sync_status::CatalogSyncStatus;
use crate::models::municipio::{Column as MunicipioColumn, Entity as MunicipioEntity};
use crate::services::nfse::catalog_sync_runs;
use crate::services::nfse_anexo_a_import;
use crate::settings;

pub async fn run_nacional_import(
    db: &doido::model::sea_orm::DatabaseConnection,
) -> doido_core::Result<serde_json::Value> {
    let cfg = settings::get();
    let anexo_a = cfg.nfse_catalog.anexo_a_url.as_deref();
    let anexo_a_stats = nfse_anexo_a_import::run(db, anexo_a).await?;

    let incidencia = cfg.nfse_catalog.incidencia_xlsx_url.as_deref();
    let inc_stats = crate::services::nfse::import_incidencia_cna(db, incidencia).await?;

    let anexo_b = cfg.nfse_catalog.anexo_b_url.as_deref();
    let b_stats = crate::services::nfse::import_catalogo_nacional(db, anexo_b).await?;

    Ok(json!({
        "anexo_a": {
            "municipios_upserted": anexo_a_stats.municipios_upserted,
            "municipios_skipped": anexo_a_stats.municipios_skipped,
            "paises_upserted": anexo_a_stats.paises_upserted,
            "paises_skipped": anexo_a_stats.paises_skipped,
        },
        "incidencia": {
            "upserted": inc_stats.upserted,
            "skipped": inc_stats.skipped,
        },
        "anexo_b": {
            "upserted": b_stats.upserted,
            "skipped": b_stats.skipped,
        },
    }))
}

pub async fn list_all_ibge_codes(
    db: &doido::model::sea_orm::DatabaseConnection,
) -> doido_core::Result<Vec<String>> {
    let rows = MunicipioEntity::find()
        .select_only()
        .column(MunicipioColumn::CodigoIbge)
        .into_tuple::<String>()
        .all(db)
        .await
        .map_err(|e| doido_core::anyhow::anyhow!("list municipios: {e}"))?;
    Ok(rows)
}

pub fn chunk_ibge(codes: &[String], batch_size: u32) -> Vec<Vec<String>> {
    let size = batch_size.max(1) as usize;
    codes.chunks(size).map(|c| c.to_vec()).collect()
}

pub async fn record_nacional_run(
    db: &doido::model::sea_orm::DatabaseConnection,
    stats: serde_json::Value,
) -> doido_core::Result<()> {
    let id = catalog_sync_runs::start_run(db, CatalogSyncKind::Nacional).await?;
    catalog_sync_runs::finish_run(db, id, CatalogSyncStatus::Ok, Some(stats.to_string())).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_ibge_splits_batches() {
        let codes: Vec<String> = (0..5).map(|i| format!("{i:07}")).collect();
        let batches = chunk_ibge(&codes, 2);
        assert_eq!(batches.len(), 3);
        assert_eq!(batches[0].len(), 2);
    }
}

pub async fn record_municipios_run(
    db: &doido::model::sea_orm::DatabaseConnection,
    stats: serde_json::Value,
    status: CatalogSyncStatus,
) -> doido_core::Result<()> {
    let id = catalog_sync_runs::start_run(db, CatalogSyncKind::Municipios).await?;
    catalog_sync_runs::finish_run(db, id, status, Some(stats.to_string())).await
}
