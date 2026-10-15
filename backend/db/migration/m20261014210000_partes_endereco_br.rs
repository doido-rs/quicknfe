use doido::model::sea_orm::ConnectionTrait;
use doido::model::sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261014210000_partes_endereco_br"
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
            "ALTER TABLE empresas ADD COLUMN cep VARCHAR(8) NULL AFTER municipio_id",
        )
        .await?;
        exec(
            manager,
            "ALTER TABLE empresas ADD COLUMN logradouro VARCHAR(255) NULL AFTER cep",
        )
        .await?;
        exec(
            manager,
            "ALTER TABLE empresas ADD COLUMN numero VARCHAR(20) NULL AFTER logradouro",
        )
        .await?;
        exec(
            manager,
            "ALTER TABLE empresas ADD COLUMN bairro VARCHAR(100) NULL AFTER numero",
        )
        .await?;
        exec(
            manager,
            "ALTER TABLE clientes ADD COLUMN cep VARCHAR(8) NULL AFTER bairro",
        )
        .await?;
        exec(
            manager,
            "ALTER TABLE intermediarios ADD COLUMN municipio_id BIGINT NULL AFTER telefone",
        )
        .await?;
        exec(
            manager,
            "ALTER TABLE intermediarios ADD COLUMN cep VARCHAR(8) NULL AFTER municipio_id",
        )
        .await?;
        exec(
            manager,
            "ALTER TABLE intermediarios ADD COLUMN logradouro VARCHAR(255) NULL AFTER cep",
        )
        .await?;
        exec(
            manager,
            "ALTER TABLE intermediarios ADD COLUMN numero VARCHAR(20) NULL AFTER logradouro",
        )
        .await?;
        exec(
            manager,
            "ALTER TABLE intermediarios ADD COLUMN bairro VARCHAR(100) NULL AFTER numero",
        )
        .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for sql in [
            "ALTER TABLE intermediarios DROP COLUMN bairro",
            "ALTER TABLE intermediarios DROP COLUMN numero",
            "ALTER TABLE intermediarios DROP COLUMN logradouro",
            "ALTER TABLE intermediarios DROP COLUMN cep",
            "ALTER TABLE intermediarios DROP COLUMN municipio_id",
            "ALTER TABLE clientes DROP COLUMN cep",
            "ALTER TABLE empresas DROP COLUMN bairro",
            "ALTER TABLE empresas DROP COLUMN numero",
            "ALTER TABLE empresas DROP COLUMN logradouro",
            "ALTER TABLE empresas DROP COLUMN cep",
        ] {
            exec(manager, sql).await?;
        }
        Ok(())
    }
}
