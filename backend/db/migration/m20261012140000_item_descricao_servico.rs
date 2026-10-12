use doido::model::sea_orm::ConnectionTrait;
use doido::model::sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261012140000_item_descricao_servico"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE itens_documento_fiscal
                    ADD COLUMN descricao_servico TEXT NULL AFTER descricao",
            )
            .await?;
        manager
            .get_connection()
            .execute_unprepared(
                "UPDATE itens_documento_fiscal
                    SET descricao_servico = descricao
                    WHERE descricao_servico IS NULL",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE itens_documento_fiscal DROP COLUMN descricao_servico",
            )
            .await?;
        Ok(())
    }
}
