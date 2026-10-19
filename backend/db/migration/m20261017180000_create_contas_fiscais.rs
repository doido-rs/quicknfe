use doido::model::migration::{create_table, drop_table};
use doido::model::sea_orm::ConnectionTrait;
use doido::model::sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261017180000_create_contas_fiscais"
    }
}

async fn exec(manager: &SchemaManager<'_>, sql: &str) -> Result<(), DbErr> {
    manager.get_connection().execute_unprepared(sql).await?;
    Ok(())
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_table(manager, "contas_fiscais", |t| {
            t.big_integer("empresa_id").not_null();
            t.string("codigo").not_null();
            t.string("descricao");
            t.big_integer("cna_id").not_null();
            t.string("codigo_tributacao_nacional").not_null();
            t.string("codigo_tributacao_municipal").not_null();
        })
        .await?;

        exec(
            manager,
            "ALTER TABLE contas_fiscais
                ADD CONSTRAINT fk_contas_fiscais_empresa_id
                FOREIGN KEY (empresa_id) REFERENCES empresas(id) ON DELETE CASCADE",
        )
        .await?;

        exec(
            manager,
            "ALTER TABLE contas_fiscais
                ADD CONSTRAINT fk_contas_fiscais_cna_id
                FOREIGN KEY (cna_id) REFERENCES cnas(id)",
        )
        .await?;

        exec(
            manager,
            "CREATE UNIQUE INDEX uq_contas_fiscais_empresa_codigo
                ON contas_fiscais (empresa_id, codigo)",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        drop_table(manager, "contas_fiscais").await
    }
}
