use doido::model::migration::{create_table, drop_table};
use doido::model::sea_orm::ConnectionTrait;
use doido::model::sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261012010000_create_paises"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_table(manager, "paises", |t| {
            t.string("codigo_iso2").not_null().unique_key();
            t.string("nome").not_null();
        })
        .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE paises
                    ADD CONSTRAINT chk_paises_codigo_iso2_len
                    CHECK (CHAR_LENGTH(codigo_iso2) = 2)",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        drop_table(manager, "paises").await?;
        Ok(())
    }
}
