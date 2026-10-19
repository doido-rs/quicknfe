use doido::model::sea_orm::ConnectionTrait;
use doido::model::sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261017180100_item_conta_fiscal_municipal"
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
            "ALTER TABLE itens_documento_fiscal
                ADD COLUMN codigo_tributacao_municipal VARCHAR(3) NULL AFTER codigo_tributacao_nacional",
        )
        .await?;
        exec(
            manager,
            "ALTER TABLE itens_documento_fiscal
                ADD COLUMN conta_fiscal_id BIGINT NULL AFTER cna_id",
        )
        .await?;
        exec(
            manager,
            "ALTER TABLE itens_documento_fiscal
                ADD CONSTRAINT fk_itens_conta_fiscal_id
                FOREIGN KEY (conta_fiscal_id) REFERENCES contas_fiscais(id) ON DELETE SET NULL",
        )
        .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        exec(
            manager,
            "ALTER TABLE itens_documento_fiscal DROP FOREIGN KEY fk_itens_conta_fiscal_id",
        )
        .await?;
        exec(
            manager,
            "ALTER TABLE itens_documento_fiscal DROP COLUMN conta_fiscal_id",
        )
        .await?;
        exec(
            manager,
            "ALTER TABLE itens_documento_fiscal DROP COLUMN codigo_tributacao_municipal",
        )
        .await?;
        Ok(())
    }
}
