use doido::model::sea_orm::ConnectionTrait;
use doido::model::sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261013150000_seed_municipio_rio_de_janeiro"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "INSERT INTO municipios (codigo_ibge, nome, uf)
                    VALUES ('3304557', 'Rio de Janeiro', 'RJ')
                    ON DUPLICATE KEY UPDATE
                        nome = 'Rio de Janeiro',
                        uf = 'RJ'",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DELETE FROM municipios
                    WHERE codigo_ibge = '3304557'
                      AND NOT EXISTS (SELECT 1 FROM empresas e WHERE e.municipio_id = municipios.id)
                      AND NOT EXISTS (SELECT 1 FROM clientes c WHERE c.municipio_id = municipios.id)
                      AND NOT EXISTS (
                          SELECT 1 FROM documentos_fiscais d
                          WHERE d.municipio_emissao_id = municipios.id
                             OR d.municipio_prestacao_id = municipios.id
                      )",
            )
            .await?;

        Ok(())
    }
}
