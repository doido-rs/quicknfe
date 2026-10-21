use doido::model::sea_orm::ConnectionTrait;
use doido::model::sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261020180000_integer_categorical_columns"
    }
}

async fn exec(manager: &SchemaManager<'_>, sql: &str) -> Result<(), DbErr> {
    manager.get_connection().execute_unprepared(sql).await?;
    Ok(())
}

async fn drop_check_if_exists(manager: &SchemaManager<'_>, table: &str, name: &str) {
    let _ = exec(
        manager,
        &format!("ALTER TABLE {table} DROP CHECK {name}"),
    )
    .await;
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        drop_check_if_exists(manager, "empresas", "chk_empresas_regime_tributario").await;
        drop_check_if_exists(manager, "companies_usuarios", "chk_companies_usuarios_papel").await;
        drop_check_if_exists(manager, "certificados_digitais", "chk_certificados_tipo").await;

        exec(
            manager,
            "ALTER TABLE empresas ADD COLUMN regime_tributario_int INTEGER NULL AFTER regime_tributario",
        )
        .await?;
        exec(
            manager,
            "UPDATE empresas SET regime_tributario_int = CASE UPPER(regime_tributario)
                WHEN 'MICROEMPREENDEDOR_INDIVIDUAL' THEN 1
                WHEN 'SIMPLES_NACIONAL' THEN 2
                WHEN 'SIMPLES_NACIONAL_EXCESSO_SUBLIMITE' THEN 3
                WHEN 'LUCRO_PRESUMIDO' THEN 4
                WHEN 'LUCRO_REAL' THEN 5
                WHEN 'LUCRO_ARBITRADO' THEN 6
                WHEN 'REGIME_NORMAL' THEN 7
                WHEN 'IMUNIDADE' THEN 8
                WHEN 'ISENCAO' THEN 9
                WHEN '1' THEN 2
                WHEN '2' THEN 3
                WHEN '3' THEN 7
                ELSE 7
            END",
        )
        .await?;
        exec(manager, "ALTER TABLE empresas DROP COLUMN regime_tributario").await?;
        exec(
            manager,
            "ALTER TABLE empresas CHANGE regime_tributario_int regime_tributario INTEGER NOT NULL",
        )
        .await?;

        exec(
            manager,
            "ALTER TABLE companies_usuarios ADD COLUMN papel_int INTEGER NULL AFTER papel",
        )
        .await?;
        exec(
            manager,
            "UPDATE companies_usuarios SET papel_int = CASE LOWER(papel)
                WHEN 'proprietario' THEN 1
                WHEN 'admin' THEN 2
                WHEN 'membro' THEN 3
                WHEN 'leitura' THEN 4
                ELSE 3
            END",
        )
        .await?;
        exec(manager, "ALTER TABLE companies_usuarios DROP COLUMN papel").await?;
        exec(
            manager,
            "ALTER TABLE companies_usuarios CHANGE papel_int papel INTEGER NOT NULL",
        )
        .await?;

        exec(
            manager,
            "ALTER TABLE certificados_digitais ADD COLUMN tipo_int INTEGER NULL AFTER tipo",
        )
        .await?;
        exec(
            manager,
            "UPDATE certificados_digitais SET tipo_int = CASE UPPER(tipo)
                WHEN 'A1' THEN 1
                WHEN 'A3' THEN 2
                WHEN 'PFX' THEN 3
                WHEN 'PEM' THEN 4
                ELSE 1
            END",
        )
        .await?;
        exec(manager, "ALTER TABLE certificados_digitais DROP COLUMN tipo").await?;
        exec(
            manager,
            "ALTER TABLE certificados_digitais CHANGE tipo_int tipo INTEGER NOT NULL",
        )
        .await?;

        exec(
            manager,
            "ALTER TABLE users ADD COLUMN date_format_int INTEGER NULL AFTER date_format",
        )
        .await?;
        exec(
            manager,
            "UPDATE users SET date_format_int = CASE UPPER(date_format)
                WHEN 'DMY' THEN 1
                WHEN 'MDY' THEN 2
                WHEN 'YMD' THEN 3
                ELSE 1
            END",
        )
        .await?;
        exec(manager, "ALTER TABLE users DROP COLUMN date_format").await?;
        exec(
            manager,
            "ALTER TABLE users CHANGE date_format_int date_format INTEGER NOT NULL DEFAULT 1",
        )
        .await?;

        exec(
            manager,
            "ALTER TABLE documentos_fiscais ADD COLUMN status_int INTEGER NULL AFTER status",
        )
        .await?;
        exec(
            manager,
            "UPDATE documentos_fiscais SET status_int = CASE LOWER(status)
                WHEN 'novo' THEN 1
                WHEN 'rascunho' THEN 2
                WHEN 'assinando' THEN 3
                WHEN 'assinado' THEN 4
                WHEN 'transmitindo' THEN 5
                WHEN 'autorizada' THEN 6
                WHEN 'autorizado' THEN 6
                WHEN 'rejeitada' THEN 7
                WHEN 'erro_assinatura' THEN 8
                WHEN 'erro_transmissao' THEN 9
                ELSE 1
            END",
        )
        .await?;
        exec(manager, "ALTER TABLE documentos_fiscais DROP COLUMN status").await?;
        exec(
            manager,
            "ALTER TABLE documentos_fiscais CHANGE status_int status INTEGER NOT NULL",
        )
        .await?;

        for col in ["tomador_localidade", "intermediario_localidade"] {
            exec(
                manager,
                &format!(
                    "ALTER TABLE documentos_fiscais ADD COLUMN {col}_int INTEGER NULL AFTER {col}"
                ),
            )
            .await?;
            exec(
                manager,
                &format!(
                    "UPDATE documentos_fiscais SET {col}_int = CASE LOWER({col})
                        WHEN 'brasil' THEN 1
                        WHEN 'exterior' THEN 2
                        WHEN 'nao_informado' THEN 3
                        ELSE 1
                    END"
                ),
            )
            .await?;
            exec(
                manager,
                &format!("ALTER TABLE documentos_fiscais DROP COLUMN {col}"),
            )
            .await?;
            exec(
                manager,
                &format!(
                    "ALTER TABLE documentos_fiscais CHANGE {col}_int {col} INTEGER NOT NULL"
                ),
            )
            .await?;
        }

        exec(
            manager,
            "ALTER TABLE documentos_fiscais ADD COLUMN trib_issqn_int INTEGER NULL AFTER trib_issqn",
        )
        .await?;
        exec(
            manager,
            "UPDATE documentos_fiscais SET trib_issqn_int = CAST(trib_issqn AS UNSIGNED)",
        )
        .await?;
        exec(manager, "ALTER TABLE documentos_fiscais DROP COLUMN trib_issqn").await?;
        exec(
            manager,
            "ALTER TABLE documentos_fiscais CHANGE trib_issqn_int trib_issqn INTEGER NOT NULL",
        )
        .await?;

        exec(
            manager,
            "ALTER TABLE documentos_fiscais ADD COLUMN reg_esp_trib_issqn_int INTEGER NULL AFTER reg_esp_trib_issqn",
        )
        .await?;
        exec(
            manager,
            "UPDATE documentos_fiscais SET reg_esp_trib_issqn_int = CAST(reg_esp_trib_issqn AS UNSIGNED)",
        )
        .await?;
        exec(
            manager,
            "ALTER TABLE documentos_fiscais DROP COLUMN reg_esp_trib_issqn",
        )
        .await?;
        exec(
            manager,
            "ALTER TABLE documentos_fiscais CHANGE reg_esp_trib_issqn_int reg_esp_trib_issqn INTEGER NOT NULL",
        )
        .await?;

        exec(
            manager,
            "ALTER TABLE documentos_fiscais ADD COLUMN issqn_tp_suspensao_int INTEGER NULL AFTER issqn_tp_suspensao",
        )
        .await?;
        exec(
            manager,
            "UPDATE documentos_fiscais SET issqn_tp_suspensao_int = CAST(issqn_tp_suspensao AS UNSIGNED)",
        )
        .await?;
        exec(
            manager,
            "ALTER TABLE documentos_fiscais DROP COLUMN issqn_tp_suspensao",
        )
        .await?;
        exec(
            manager,
            "ALTER TABLE documentos_fiscais CHANGE issqn_tp_suspensao_int issqn_tp_suspensao INTEGER NOT NULL DEFAULT 1",
        )
        .await?;

        exec(
            manager,
            "ALTER TABLE documentos_fiscais ADD COLUMN cst_pis_cofins_int INTEGER NULL AFTER cst_pis_cofins",
        )
        .await?;
        exec(
            manager,
            "UPDATE documentos_fiscais SET cst_pis_cofins_int = CAST(cst_pis_cofins AS UNSIGNED)",
        )
        .await?;
        exec(
            manager,
            "ALTER TABLE documentos_fiscais DROP COLUMN cst_pis_cofins",
        )
        .await?;
        exec(
            manager,
            "ALTER TABLE documentos_fiscais CHANGE cst_pis_cofins_int cst_pis_cofins INTEGER NOT NULL",
        )
        .await?;

        exec(
            manager,
            "ALTER TABLE documentos_fiscais ADD COLUMN tp_ret_pis_cofins_csll_int INTEGER NULL AFTER tp_ret_pis_cofins_csll",
        )
        .await?;
        exec(
            manager,
            "UPDATE documentos_fiscais SET tp_ret_pis_cofins_csll_int = CAST(tp_ret_pis_cofins_csll AS UNSIGNED)",
        )
        .await?;
        exec(
            manager,
            "ALTER TABLE documentos_fiscais DROP COLUMN tp_ret_pis_cofins_csll",
        )
        .await?;
        exec(
            manager,
            "ALTER TABLE documentos_fiscais CHANGE tp_ret_pis_cofins_csll_int tp_ret_pis_cofins_csll INTEGER NOT NULL",
        )
        .await?;

        exec(
            manager,
            "ALTER TABLE documentos_fiscais ADD COLUMN modelo_int INTEGER NULL AFTER modelo",
        )
        .await?;
        exec(
            manager,
            "UPDATE documentos_fiscais SET modelo_int = 1",
        )
        .await?;
        exec(manager, "ALTER TABLE documentos_fiscais DROP COLUMN modelo").await?;
        exec(
            manager,
            "ALTER TABLE documentos_fiscais CHANGE modelo_int modelo INTEGER NOT NULL",
        )
        .await?;

        exec(
            manager,
            "ALTER TABLE lotes_transmissao ADD COLUMN status_lote_int INTEGER NULL AFTER status_lote",
        )
        .await?;
        exec(
            manager,
            "UPDATE lotes_transmissao SET status_lote_int = CASE LOWER(status_lote)
                WHEN 'aberto' THEN 1
                WHEN 'enviado' THEN 2
                WHEN 'processado' THEN 3
                WHEN 'erro' THEN 4
                ELSE 3
            END",
        )
        .await?;
        exec(manager, "ALTER TABLE lotes_transmissao DROP COLUMN status_lote").await?;
        exec(
            manager,
            "ALTER TABLE lotes_transmissao CHANGE status_lote_int status_lote INTEGER NOT NULL",
        )
        .await?;

        exec(
            manager,
            "ALTER TABLE eventos_documento_fiscal ADD COLUMN tipo_evento_int INTEGER NULL AFTER tipo_evento",
        )
        .await?;
        exec(
            manager,
            "UPDATE eventos_documento_fiscal SET tipo_evento_int = CASE LOWER(tipo_evento)
                WHEN 'cancelamento' THEN 1
                WHEN 'substituicao' THEN 2
                ELSE 3
            END",
        )
        .await?;
        exec(
            manager,
            "ALTER TABLE eventos_documento_fiscal DROP COLUMN tipo_evento",
        )
        .await?;
        exec(
            manager,
            "ALTER TABLE eventos_documento_fiscal CHANGE tipo_evento_int tipo_evento INTEGER NOT NULL",
        )
        .await?;

        exec(
            manager,
            "ALTER TABLE tributos_item ADD COLUMN tipo_imposto_int INTEGER NULL AFTER tipo_imposto",
        )
        .await?;
        exec(
            manager,
            "UPDATE tributos_item SET tipo_imposto_int = CASE UPPER(tipo_imposto)
                WHEN 'ISS' THEN 1
                WHEN 'PIS' THEN 2
                WHEN 'COFINS' THEN 3
                WHEN 'CSLL' THEN 4
                WHEN 'IRRF' THEN 5
                WHEN 'INSS' THEN 6
                ELSE 7
            END",
        )
        .await?;
        exec(manager, "ALTER TABLE tributos_item DROP COLUMN tipo_imposto").await?;
        exec(
            manager,
            "ALTER TABLE tributos_item CHANGE tipo_imposto_int tipo_imposto INTEGER NOT NULL",
        )
        .await?;

        exec(
            manager,
            "ALTER TABLE nfse_catalog_sync_runs ADD COLUMN kind_int INTEGER NULL AFTER kind",
        )
        .await?;
        exec(
            manager,
            "UPDATE nfse_catalog_sync_runs SET kind_int = CASE LOWER(kind)
                WHEN 'nacional' THEN 1
                WHEN 'municipios' THEN 2
                ELSE 3
            END",
        )
        .await?;
        exec(manager, "ALTER TABLE nfse_catalog_sync_runs DROP COLUMN kind").await?;
        exec(
            manager,
            "ALTER TABLE nfse_catalog_sync_runs CHANGE kind_int kind INTEGER NOT NULL",
        )
        .await?;

        exec(
            manager,
            "ALTER TABLE nfse_catalog_sync_runs ADD COLUMN status_int INTEGER NULL AFTER status",
        )
        .await?;
        exec(
            manager,
            "UPDATE nfse_catalog_sync_runs SET status_int = CASE LOWER(status)
                WHEN 'running' THEN 1
                WHEN 'ok' THEN 2
                WHEN 'error' THEN 3
                WHEN 'erro' THEN 3
                ELSE 2
            END",
        )
        .await?;
        exec(manager, "ALTER TABLE nfse_catalog_sync_runs DROP COLUMN status").await?;
        exec(
            manager,
            "ALTER TABLE nfse_catalog_sync_runs CHANGE status_int status INTEGER NOT NULL",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Custom(
            "integer categorical columns migration is not reversible automatically".into(),
        ))
    }
}
