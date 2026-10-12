use doido::model::sea_orm::ConnectionTrait;
use doido::model::sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261012160000_documento_info_complementares"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE documentos_fiscais
                    ADD COLUMN numero_doc_responsabilidade_tecnica TEXT NULL AFTER xml_processado,
                    ADD COLUMN documento_referencia TEXT NULL AFTER numero_doc_responsabilidade_tecnica,
                    ADD COLUMN informacoes_complementares TEXT NULL AFTER documento_referencia,
                    ADD COLUMN numero_pedido_ordem_b2b TEXT NULL AFTER informacoes_complementares",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE documentos_fiscais
                    DROP COLUMN numero_pedido_ordem_b2b,
                    DROP COLUMN informacoes_complementares,
                    DROP COLUMN documento_referencia,
                    DROP COLUMN numero_doc_responsabilidade_tecnica",
            )
            .await?;
        Ok(())
    }
}
