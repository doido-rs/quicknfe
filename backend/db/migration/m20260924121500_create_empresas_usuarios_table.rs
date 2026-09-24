use doido::model::migration::{create_table, drop_table};
use doido::model::sea_orm::ConnectionTrait;
use doido::model::sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260924121500_create_empresas_usuarios_table"
    }
}

async fn exec(manager: &SchemaManager<'_>, sql: &str) -> Result<(), DbErr> {
    manager.get_connection().execute_unprepared(sql).await?;
    Ok(())
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_table(manager, "empresas_usuarios", |t| {
            t.big_integer("user_id").not_null();
            t.big_integer("empresa_id").not_null();
            t.string("papel").not_null().default("membro");
            t.timestamp("criado_em")
                .not_null()
                .default(Expr::current_timestamp());
        })
        .await?;

        exec(
            manager,
            "ALTER TABLE empresas_usuarios
                ADD CONSTRAINT chk_empresas_usuarios_papel
                CHECK (papel IN ('proprietario', 'admin', 'membro', 'leitura'))",
        )
        .await?;

        exec(
            manager,
            "CREATE UNIQUE INDEX unq_empresas_usuarios_user_empresa
                ON empresas_usuarios (user_id, empresa_id)",
        )
        .await?;

        exec(
            manager,
            "ALTER TABLE empresas_usuarios
                ADD CONSTRAINT fk_empresas_usuarios_user_id
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE",
        )
        .await?;

        exec(
            manager,
            "ALTER TABLE empresas_usuarios
                ADD CONSTRAINT fk_empresas_usuarios_empresa_id
                FOREIGN KEY (empresa_id) REFERENCES empresas(id) ON DELETE CASCADE",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        drop_table(manager, "empresas_usuarios").await
    }
}
