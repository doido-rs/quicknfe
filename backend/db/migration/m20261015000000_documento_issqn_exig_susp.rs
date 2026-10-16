use doido::model::sea_orm::ConnectionTrait;
use doido::model::sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261015000000_documento_issqn_exig_susp"
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
            "ALTER TABLE documentos_fiscais
                ADD COLUMN issqn_tp_suspensao VARCHAR(1) NOT NULL DEFAULT '1'
                    AFTER issqn_exigibilidade_suspensa,
                ADD COLUMN issqn_n_processo_suspensao VARCHAR(30) NULL
                    AFTER issqn_tp_suspensao",
        )
        .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        exec(
            manager,
            "ALTER TABLE documentos_fiscais
                DROP COLUMN issqn_n_processo_suspensao,
                DROP COLUMN issqn_tp_suspensao",
        )
        .await
    }
}
