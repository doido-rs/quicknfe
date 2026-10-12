use doido::model::sea_orm::ConnectionTrait;
use doido::model::sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261012190000_documento_tributacao_federal"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE documentos_fiscais
                    ADD COLUMN cst_pis_cofins VARCHAR(2) NOT NULL DEFAULT '00'
                        AFTER issqn_valor,
                    ADD COLUMN tp_ret_pis_cofins_csll VARCHAR(2) NOT NULL DEFAULT '0'
                        AFTER cst_pis_cofins,
                    ADD COLUMN valor_irrf DECIMAL(15, 2) NULL
                        AFTER tp_ret_pis_cofins_csll,
                    ADD COLUMN valor_contrib_sociais_retidas DECIMAL(15, 2) NULL
                        AFTER valor_irrf,
                    ADD COLUMN valor_contrib_previdenciaria_retida DECIMAL(15, 2) NULL
                        AFTER valor_contrib_sociais_retidas",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE documentos_fiscais
                    DROP COLUMN valor_contrib_previdenciaria_retida,
                    DROP COLUMN valor_contrib_sociais_retidas,
                    DROP COLUMN valor_irrf,
                    DROP COLUMN tp_ret_pis_cofins_csll,
                    DROP COLUMN cst_pis_cofins",
            )
            .await?;
        Ok(())
    }
}
