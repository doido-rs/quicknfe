use doido::model::sea_orm::ConnectionTrait;
use doido::model::sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261011210000_documento_partes_localidade"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE documentos_fiscais
                    ADD COLUMN tomador_localidade VARCHAR(20) NOT NULL DEFAULT 'nao_informado'
                        AFTER intermediario_id,
                    ADD COLUMN intermediario_localidade VARCHAR(20) NOT NULL DEFAULT 'nao_informado'
                        AFTER tomador_localidade,
                    ADD COLUMN tomador_exterior TEXT NULL AFTER intermediario_localidade,
                    ADD COLUMN intermediario_exterior TEXT NULL AFTER tomador_exterior",
            )
            .await?;
        manager
            .get_connection()
            .execute_unprepared(
                "UPDATE documentos_fiscais SET tomador_localidade = 'brasil' WHERE cliente_id IS NOT NULL",
            )
            .await?;
        manager
            .get_connection()
            .execute_unprepared(
                "UPDATE documentos_fiscais SET intermediario_localidade = 'brasil'
                    WHERE intermediario_id IS NOT NULL",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE documentos_fiscais
                    DROP COLUMN intermediario_exterior,
                    DROP COLUMN tomador_exterior,
                    DROP COLUMN intermediario_localidade,
                    DROP COLUMN tomador_localidade",
            )
            .await?;
        Ok(())
    }
}
