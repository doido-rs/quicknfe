use doido::model::migration::{create_table, drop_table};
use doido::model::sea_orm::ConnectionTrait;
use doido::model::sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261016140000_nfse_catalog_sync"
    }
}

async fn exec(manager: &SchemaManager<'_>, sql: &str) -> Result<(), DbErr> {
    manager.get_connection().execute_unprepared(sql).await?;
    Ok(())
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        exec(
            manager,
            "ALTER TABLE cnas
                ADD COLUMN incid_ep TINYINT(1) NOT NULL DEFAULT 0 AFTER descricao,
                ADD COLUMN incid_lp TINYINT(1) NOT NULL DEFAULT 0 AFTER incid_ep,
                ADD COLUMN incid_et TINYINT(1) NOT NULL DEFAULT 0 AFTER incid_lp,
                ADD COLUMN grupo_obra TINYINT(1) NOT NULL DEFAULT 0 AFTER incid_et,
                ADD COLUMN grupo_atv_evento TINYINT(1) NOT NULL DEFAULT 0 AFTER grupo_obra,
                ADD COLUMN synced_at DATETIME(6) NULL AFTER grupo_atv_evento",
        )
        .await?;

        exec(
            manager,
            "CREATE TABLE IF NOT EXISTS municipio_cna_parametrizacao (
                id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
                municipio_id BIGINT NOT NULL,
                cna_id BIGINT NULL,
                codigo_trib_nac VARCHAR(6) NOT NULL,
                codigo_trib_mun VARCHAR(3) NOT NULL,
                codigo_parametrizacao VARCHAR(32) NOT NULL,
                aliquota DECIMAL(15,4) NULL,
                vigencia_inicio DATE NULL,
                vigencia_fim DATE NULL,
                synced_at DATETIME(6) NOT NULL,
                CONSTRAINT uq_municipio_cna_param
                    UNIQUE (municipio_id, codigo_trib_nac, codigo_trib_mun),
                INDEX idx_mcp_municipio_trib (municipio_id, codigo_trib_nac),
                CONSTRAINT fk_mcp_municipio
                    FOREIGN KEY (municipio_id) REFERENCES municipios(id),
                CONSTRAINT fk_mcp_cna
                    FOREIGN KEY (cna_id) REFERENCES cnas(id)
            )",
        )
        .await?;

        create_table(manager, "nfse_catalog_sync_runs", |t| {
            t.string("kind").not_null();
            t.timestamp("started_at").not_null();
            t.timestamp("finished_at");
            t.string("status").not_null();
            t.text("stats_json");
        })
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        drop_table(manager, "nfse_catalog_sync_runs").await?;
        drop_table(manager, "municipio_cna_parametrizacao").await?;
        exec(
            manager,
            "ALTER TABLE cnas
                DROP COLUMN incid_ep,
                DROP COLUMN incid_lp,
                DROP COLUMN incid_et,
                DROP COLUMN grupo_obra,
                DROP COLUMN grupo_atv_evento,
                DROP COLUMN synced_at",
        )
        .await?;
        Ok(())
    }
}
