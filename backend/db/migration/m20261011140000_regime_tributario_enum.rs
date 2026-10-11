use doido::model::sea_orm::ConnectionTrait;
use doido::model::sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261011140000_regime_tributario_enum"
    }
}

async fn exec(manager: &SchemaManager<'_>, sql: &str) -> Result<(), DbErr> {
    manager.get_connection().execute_unprepared(sql).await?;
    Ok(())
}

const REGIMES: &str = "'MICROEMPREENDEDOR_INDIVIDUAL','SIMPLES_NACIONAL','SIMPLES_NACIONAL_EXCESSO_SUBLIMITE','LUCRO_PRESUMIDO','LUCRO_REAL','LUCRO_ARBITRADO','REGIME_NORMAL','IMUNIDADE','ISENCAO'";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        exec(
            manager,
            "ALTER TABLE empresas ADD COLUMN regime_tributario_cod VARCHAR(64) NULL AFTER regime_tributario",
        )
        .await?;

        exec(
            manager,
            "UPDATE empresas SET regime_tributario_cod = CASE regime_tributario
                WHEN 1 THEN 'SIMPLES_NACIONAL'
                WHEN 2 THEN 'SIMPLES_NACIONAL_EXCESSO_SUBLIMITE'
                WHEN 3 THEN 'REGIME_NORMAL'
                ELSE 'REGIME_NORMAL'
            END",
        )
        .await?;

        exec(manager, "ALTER TABLE empresas DROP COLUMN regime_tributario").await?;

        exec(
            manager,
            "ALTER TABLE empresas CHANGE regime_tributario_cod regime_tributario VARCHAR(64) NOT NULL",
        )
        .await?;

        exec(
            manager,
            &format!(
                "ALTER TABLE empresas
                    ADD CONSTRAINT chk_empresas_regime_tributario
                    CHECK (regime_tributario IN ({REGIMES}))"
            ),
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        exec(
            manager,
            "ALTER TABLE empresas DROP CHECK chk_empresas_regime_tributario",
        )
        .await?;

        exec(
            manager,
            "ALTER TABLE empresas ADD COLUMN regime_tributario_int INTEGER NULL AFTER regime_tributario",
        )
        .await?;

        exec(
            manager,
            "UPDATE empresas SET regime_tributario_int = CASE regime_tributario
                WHEN 'SIMPLES_NACIONAL' THEN 1
                WHEN 'MICROEMPREENDEDOR_INDIVIDUAL' THEN 1
                WHEN 'SIMPLES_NACIONAL_EXCESSO_SUBLIMITE' THEN 2
                WHEN 'LUCRO_PRESUMIDO' THEN 3
                WHEN 'LUCRO_REAL' THEN 3
                WHEN 'LUCRO_ARBITRADO' THEN 3
                WHEN 'REGIME_NORMAL' THEN 3
                WHEN 'IMUNIDADE' THEN 3
                WHEN 'ISENCAO' THEN 3
                ELSE 3
            END",
        )
        .await?;

        exec(manager, "ALTER TABLE empresas DROP COLUMN regime_tributario").await?;

        exec(
            manager,
            "ALTER TABLE empresas CHANGE regime_tributario_int regime_tributario INTEGER NOT NULL",
        )
        .await?;

        Ok(())
    }
}
