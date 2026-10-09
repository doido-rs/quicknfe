use doido::model::migration::{create_table, drop_table};
use doido::model::sea_orm::ConnectionTrait;
use doido::model::sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261009180000_create_intermediarios"
    }
}

async fn exec(manager: &SchemaManager<'_>, sql: &str) -> Result<(), DbErr> {
    manager.get_connection().execute_unprepared(sql).await?;
    Ok(())
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_table(manager, "intermediarios", |t| {
            t.big_integer("empresa_id").not_null();
            t.string("cpf_cnpj").not_null();
            t.string("razao_social").not_null();
            t.string("inscricao_municipal");
            t.string("email");
            t.string("telefone");
        })
        .await?;

        exec(
            manager,
            "ALTER TABLE intermediarios
                ADD CONSTRAINT chk_intermediarios_cpf_cnpj
                CHECK (CHAR_LENGTH(cpf_cnpj) IN (11, 14))",
        )
        .await?;

        exec(
            manager,
            "ALTER TABLE intermediarios
                ADD CONSTRAINT fk_intermediarios_empresa_id
                FOREIGN KEY (empresa_id) REFERENCES empresas(id) ON DELETE CASCADE",
        )
        .await?;

        exec(
            manager,
            "ALTER TABLE documentos_fiscais
                ADD COLUMN intermediario_id BIGINT NULL AFTER cliente_id",
        )
        .await?;

        exec(
            manager,
            "ALTER TABLE documentos_fiscais
                ADD CONSTRAINT fk_documentos_fiscais_intermediario_id
                FOREIGN KEY (intermediario_id) REFERENCES intermediarios(id)",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        exec(
            manager,
            "ALTER TABLE documentos_fiscais
                DROP FOREIGN KEY fk_documentos_fiscais_intermediario_id",
        )
        .await?;
        exec(
            manager,
            "ALTER TABLE documentos_fiscais DROP COLUMN intermediario_id",
        )
        .await?;
        drop_table(manager, "intermediarios").await
    }
}
