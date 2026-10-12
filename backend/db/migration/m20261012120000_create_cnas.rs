use doido::model::migration::{create_table, drop_table};
use doido::model::sea_orm::ConnectionTrait;
use doido::model::sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261012120000_create_cnas"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_table(manager, "cnas", |t| {
            t.string("codigo").not_null().unique_key();
            t.text("descricao").not_null();
        })
        .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "INSERT INTO cnas (codigo, descricao) VALUES
                    ('010101', 'Análise e desenvolvimento de sistemas'),
                    ('010201', 'Programação'),
                    ('010301', 'Processamento de dados, textos, imagens, vídeos e congêneres'),
                    ('010401', 'Elaboração de programas de computadores, inclusive de jogos eletrônicos'),
                    ('010501', 'Licenciamento ou cessão de direito de uso de programas de computação')",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE itens_documento_fiscal
                    ADD COLUMN cna_id BIGINT NULL AFTER codigo_tributacao_nacional",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "UPDATE itens_documento_fiscal i
                    INNER JOIN cnas c ON c.codigo = i.codigo_tributacao_nacional
                    SET i.cna_id = c.id
                    WHERE i.cna_id IS NULL AND i.codigo_tributacao_nacional IS NOT NULL",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE itens_documento_fiscal
                    ADD CONSTRAINT fk_itens_cna_id
                        FOREIGN KEY (cna_id) REFERENCES cnas(id)",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE itens_documento_fiscal DROP FOREIGN KEY fk_itens_cna_id",
            )
            .await?;
        manager
            .get_connection()
            .execute_unprepared("ALTER TABLE itens_documento_fiscal DROP COLUMN cna_id")
            .await?;
        drop_table(manager, "cnas").await?;
        Ok(())
    }
}
