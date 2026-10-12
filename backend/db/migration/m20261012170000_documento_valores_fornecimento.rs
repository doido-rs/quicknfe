use doido::model::sea_orm::ConnectionTrait;
use doido::model::sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261012170000_documento_valores_fornecimento"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE documentos_fiscais
                    ADD COLUMN valor_recebido_intermediario DECIMAL(15, 2) NULL
                        AFTER valor_liquido,
                    ADD COLUMN desconto_incondicionado DECIMAL(15, 2) NOT NULL DEFAULT 0
                        AFTER valor_recebido_intermediario,
                    ADD COLUMN desconto_condicionado DECIMAL(15, 2) NOT NULL DEFAULT 0
                        AFTER desconto_incondicionado",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE documentos_fiscais
                    DROP COLUMN desconto_condicionado,
                    DROP COLUMN desconto_incondicionado,
                    DROP COLUMN valor_recebido_intermediario",
            )
            .await?;
        Ok(())
    }
}
