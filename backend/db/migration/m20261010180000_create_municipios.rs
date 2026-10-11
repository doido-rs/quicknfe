use doido::model::migration::{create_table, drop_table};
use doido::model::sea_orm::ConnectionTrait;
use doido::model::sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261010180000_create_municipios"
    }
}

async fn exec(manager: &SchemaManager<'_>, sql: &str) -> Result<(), DbErr> {
    manager.get_connection().execute_unprepared(sql).await?;
    Ok(())
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_table(manager, "municipios", |t| {
            t.string("codigo_ibge").not_null().unique_key();
            t.string("nome").not_null();
            t.string("uf").not_null();
        })
        .await?;

        exec(
            manager,
            "ALTER TABLE municipios
                ADD CONSTRAINT chk_municipios_codigo_ibge_len
                CHECK (CHAR_LENGTH(codigo_ibge) = 7)",
        )
        .await?;

        exec(
            manager,
            "ALTER TABLE municipios
                ADD CONSTRAINT chk_municipios_uf_len
                CHECK (CHAR_LENGTH(uf) = 2)",
        )
        .await?;

        exec(
            manager,
            "INSERT INTO municipios (codigo_ibge, nome, uf) VALUES ('3550308', 'São Paulo', 'SP')",
        )
        .await?;

        exec(
            manager,
            "INSERT INTO municipios (codigo_ibge, nome, uf)
                SELECT DISTINCT e.codigo_municipio_ibge,
                    CONCAT('Município IBGE ', e.codigo_municipio_ibge),
                    e.uf
                FROM empresas e
                WHERE e.codigo_municipio_ibge IS NOT NULL
                  AND e.codigo_municipio_ibge <> '3550308'",
        )
        .await?;

        exec(
            manager,
            "INSERT INTO municipios (codigo_ibge, nome, uf)
                SELECT DISTINCT c.codigo_municipio_ibge,
                    CONCAT('Município IBGE ', c.codigo_municipio_ibge),
                    COALESCE(c.uf, 'XX')
                FROM clientes c
                WHERE c.codigo_municipio_ibge IS NOT NULL
                  AND c.codigo_municipio_ibge <> ''
                  AND NOT EXISTS (
                    SELECT 1 FROM municipios m WHERE m.codigo_ibge = c.codigo_municipio_ibge
                  )",
        )
        .await?;

        exec(
            manager,
            "ALTER TABLE empresas ADD COLUMN municipio_id BIGINT NULL AFTER regime_tributario",
        )
        .await?;

        exec(
            manager,
            "ALTER TABLE clientes ADD COLUMN municipio_id BIGINT NULL AFTER bairro",
        )
        .await?;

        exec(
            manager,
            "UPDATE empresas e
                INNER JOIN municipios m ON m.codigo_ibge = e.codigo_municipio_ibge
                SET e.municipio_id = m.id",
        )
        .await?;

        exec(
            manager,
            "UPDATE clientes c
                INNER JOIN municipios m ON m.codigo_ibge = c.codigo_municipio_ibge
                SET c.municipio_id = m.id
                WHERE c.codigo_municipio_ibge IS NOT NULL AND c.codigo_municipio_ibge <> ''",
        )
        .await?;

        exec(
            manager,
            "UPDATE empresas SET municipio_id = (SELECT id FROM municipios WHERE codigo_ibge = '3550308' LIMIT 1)
                WHERE municipio_id IS NULL",
        )
        .await?;

        exec(
            manager,
            "ALTER TABLE empresas MODIFY municipio_id BIGINT NOT NULL",
        )
        .await?;

        for sql in [
            "ALTER TABLE empresas
                ADD CONSTRAINT fk_empresas_municipio_id
                FOREIGN KEY (municipio_id) REFERENCES municipios(id)",
            "ALTER TABLE clientes
                ADD CONSTRAINT fk_clientes_municipio_id
                FOREIGN KEY (municipio_id) REFERENCES municipios(id) ON DELETE SET NULL",
            "ALTER TABLE empresas DROP COLUMN codigo_municipio_ibge",
            "ALTER TABLE empresas DROP COLUMN uf",
            "ALTER TABLE clientes DROP COLUMN codigo_municipio_ibge",
            "ALTER TABLE clientes DROP COLUMN uf",
        ] {
            exec(manager, sql).await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        exec(
            manager,
            "ALTER TABLE empresas ADD COLUMN codigo_municipio_ibge VARCHAR(255) NULL",
        )
        .await?;
        exec(manager, "ALTER TABLE empresas ADD COLUMN uf VARCHAR(255) NULL").await?;
        exec(
            manager,
            "ALTER TABLE clientes ADD COLUMN codigo_municipio_ibge VARCHAR(255) NULL",
        )
        .await?;
        exec(manager, "ALTER TABLE clientes ADD COLUMN uf VARCHAR(255) NULL").await?;

        exec(
            manager,
            "UPDATE empresas e
                INNER JOIN municipios m ON m.id = e.municipio_id
                SET e.codigo_municipio_ibge = m.codigo_ibge, e.uf = m.uf",
        )
        .await?;

        exec(
            manager,
            "UPDATE clientes c
                INNER JOIN municipios m ON m.id = c.municipio_id
                SET c.codigo_municipio_ibge = m.codigo_ibge, c.uf = m.uf",
        )
        .await?;

        exec(
            manager,
            "ALTER TABLE empresas MODIFY codigo_municipio_ibge VARCHAR(255) NOT NULL",
        )
        .await?;
        exec(
            manager,
            "ALTER TABLE empresas MODIFY uf VARCHAR(255) NOT NULL",
        )
        .await?;

        exec(
            manager,
            "ALTER TABLE empresas DROP FOREIGN KEY fk_empresas_municipio_id",
        )
        .await?;
        exec(
            manager,
            "ALTER TABLE clientes DROP FOREIGN KEY fk_clientes_municipio_id",
        )
        .await?;

        exec(manager, "ALTER TABLE empresas DROP COLUMN municipio_id").await?;
        exec(manager, "ALTER TABLE clientes DROP COLUMN municipio_id").await?;

        drop_table(manager, "municipios").await
    }
}
