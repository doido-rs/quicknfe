use doido::model::sea_orm::ConnectionTrait;
use doido::model::sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261011230000_documento_municipios"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE documentos_fiscais
                    ADD COLUMN municipio_emissao_id BIGINT NULL AFTER codigo_municipio_emissao,
                    ADD COLUMN municipio_prestacao_id BIGINT NULL AFTER municipio_emissao_id",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "UPDATE documentos_fiscais d
                    INNER JOIN municipios m ON m.codigo_ibge = d.codigo_municipio_emissao
                    SET d.municipio_emissao_id = m.id,
                        d.municipio_prestacao_id = m.id
                    WHERE d.municipio_emissao_id IS NULL",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "UPDATE documentos_fiscais d
                    INNER JOIN empresas e ON e.id = d.empresa_id
                    SET d.municipio_emissao_id = e.municipio_id
                    WHERE d.municipio_emissao_id IS NULL",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "UPDATE documentos_fiscais
                    SET municipio_prestacao_id = municipio_emissao_id
                    WHERE municipio_prestacao_id IS NULL AND municipio_emissao_id IS NOT NULL",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "UPDATE documentos_fiscais d
                    CROSS JOIN (SELECT id FROM municipios ORDER BY id LIMIT 1) m
                    SET d.municipio_emissao_id = m.id
                    WHERE d.municipio_emissao_id IS NULL",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "UPDATE documentos_fiscais
                    SET municipio_prestacao_id = municipio_emissao_id
                    WHERE municipio_prestacao_id IS NULL",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE documentos_fiscais
                    MODIFY municipio_emissao_id BIGINT NOT NULL,
                    MODIFY municipio_prestacao_id BIGINT NOT NULL",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE documentos_fiscais
                    ADD CONSTRAINT fk_documentos_municipio_emissao
                        FOREIGN KEY (municipio_emissao_id) REFERENCES municipios(id),
                    ADD CONSTRAINT fk_documentos_municipio_prestacao
                        FOREIGN KEY (municipio_prestacao_id) REFERENCES municipios(id)",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE documentos_fiscais DROP COLUMN codigo_municipio_emissao",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE documentos_fiscais
                    ADD COLUMN codigo_municipio_emissao VARCHAR(255) NOT NULL DEFAULT '3550308'
                        AFTER data_competencia",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "UPDATE documentos_fiscais d
                    INNER JOIN municipios m ON m.id = d.municipio_emissao_id
                    SET d.codigo_municipio_emissao = m.codigo_ibge",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE documentos_fiscais
                    DROP FOREIGN KEY fk_documentos_municipio_emissao,
                    DROP FOREIGN KEY fk_documentos_municipio_prestacao",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE documentos_fiscais
                    DROP COLUMN municipio_prestacao_id,
                    DROP COLUMN municipio_emissao_id",
            )
            .await?;
        Ok(())
    }
}
