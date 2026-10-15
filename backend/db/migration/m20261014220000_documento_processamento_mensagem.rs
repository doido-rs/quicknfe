use doido::model::sea_orm::ConnectionTrait;
use doido::model::sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261014220000_documento_processamento_mensagem"
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
            "ALTER TABLE documentos_fiscais ADD COLUMN processamento_mensagem TEXT NULL AFTER status",
        )
        .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        exec(
            manager,
            "ALTER TABLE documentos_fiscais DROP COLUMN processamento_mensagem",
        )
        .await
    }
}
