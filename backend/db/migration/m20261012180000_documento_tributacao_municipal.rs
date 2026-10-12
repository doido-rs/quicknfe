use doido::model::sea_orm::ConnectionTrait;
use doido::model::sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261012180000_documento_tributacao_municipal"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE documentos_fiscais
                    ADD COLUMN trib_issqn VARCHAR(8) NOT NULL DEFAULT '1'
                        AFTER desconto_condicionado,
                    ADD COLUMN reg_esp_trib_issqn VARCHAR(8) NOT NULL DEFAULT '0'
                        AFTER trib_issqn,
                    ADD COLUMN issqn_exigibilidade_suspensa TINYINT NOT NULL DEFAULT 0
                        AFTER reg_esp_trib_issqn,
                    ADD COLUMN issqn_retencao TINYINT NOT NULL DEFAULT 0
                        AFTER issqn_exigibilidade_suspensa,
                    ADD COLUMN issqn_beneficio_municipal TINYINT NOT NULL DEFAULT 0
                        AFTER issqn_retencao,
                    ADD COLUMN issqn_deducao_reducao_bc TINYINT NOT NULL DEFAULT 0
                        AFTER issqn_beneficio_municipal,
                    ADD COLUMN issqn_aliquota DECIMAL(7, 4) NULL
                        AFTER issqn_deducao_reducao_bc,
                    ADD COLUMN issqn_base_calculo DECIMAL(15, 2) NULL
                        AFTER issqn_aliquota,
                    ADD COLUMN issqn_valor DECIMAL(15, 2) NULL
                        AFTER issqn_base_calculo",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE documentos_fiscais
                    DROP COLUMN issqn_valor,
                    DROP COLUMN issqn_base_calculo,
                    DROP COLUMN issqn_aliquota,
                    DROP COLUMN issqn_deducao_reducao_bc,
                    DROP COLUMN issqn_beneficio_municipal,
                    DROP COLUMN issqn_retencao,
                    DROP COLUMN issqn_exigibilidade_suspensa,
                    DROP COLUMN reg_esp_trib_issqn,
                    DROP COLUMN trib_issqn",
            )
            .await?;
        Ok(())
    }
}
