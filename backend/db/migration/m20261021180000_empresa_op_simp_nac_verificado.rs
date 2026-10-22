use doido::model::sea_orm::ConnectionTrait;
use doido::model::sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261021180000_empresa_op_simp_nac_verificado"
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
            "ALTER TABLE empresas ADD COLUMN op_simp_nac_verificado INTEGER NULL AFTER regime_tributario",
        )
        .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        exec(
            manager,
            "ALTER TABLE empresas DROP COLUMN op_simp_nac_verificado",
        )
        .await
    }
}
