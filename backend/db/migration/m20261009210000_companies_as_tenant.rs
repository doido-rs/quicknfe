use doido::model::migration::{create_table, drop_table};
use doido::model::sea_orm::ConnectionTrait;
use doido::model::sea_orm::{DatabaseBackend, FromQueryResult, Statement};
use doido::model::sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261009210000_companies_as_tenant"
    }
}

async fn exec(manager: &SchemaManager<'_>, sql: &str) -> Result<(), DbErr> {
    manager.get_connection().execute_unprepared(sql).await?;
    Ok(())
}

#[derive(Debug, FromQueryResult)]
struct EmpresaRow {
    id: i64,
    razao_social: String,
}

#[derive(Debug, FromQueryResult)]
struct ExistsRow {
    n: i64,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();
        let already = ExistsRow::find_by_statement(Statement::from_string(
            DatabaseBackend::MySql,
            "SELECT COUNT(*) AS n FROM information_schema.tables
             WHERE table_schema = DATABASE() AND table_name = 'companies'"
                .to_string(),
        ))
        .one(conn)
        .await?;
        if already.map(|r| r.n).unwrap_or(0) > 0 {
            return Ok(());
        }

        create_table(manager, "companies", |t| {
            t.string("nome").not_null();
            t.timestamp("criado_em")
                .not_null()
                .default(Expr::current_timestamp());
        })
        .await?;

        create_table(manager, "companies_usuarios", |t| {
            t.big_integer("user_id").not_null();
            t.big_integer("company_id").not_null();
            t.string("papel").not_null().default("membro");
            t.timestamp("criado_em")
                .not_null()
                .default(Expr::current_timestamp());
        })
        .await?;

        exec(
            manager,
            "ALTER TABLE companies_usuarios
                ADD CONSTRAINT chk_companies_usuarios_papel
                CHECK (papel IN ('proprietario', 'admin', 'membro', 'leitura'))",
        )
        .await?;

        exec(
            manager,
            "CREATE UNIQUE INDEX unq_companies_usuarios_user_company
                ON companies_usuarios (user_id, company_id)",
        )
        .await?;

        exec(
            manager,
            "ALTER TABLE companies_usuarios
                ADD CONSTRAINT fk_companies_usuarios_user_id
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE",
        )
        .await?;

        exec(
            manager,
            "ALTER TABLE companies_usuarios
                ADD CONSTRAINT fk_companies_usuarios_company_id
                FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE CASCADE",
        )
        .await?;

        exec(
            manager,
            "ALTER TABLE empresas ADD COLUMN company_id BIGINT NULL AFTER id",
        )
        .await?;

        let conn = manager.get_connection();
        let empresas = EmpresaRow::find_by_statement(Statement::from_string(
            DatabaseBackend::MySql,
            "SELECT id, razao_social FROM empresas ORDER BY id".to_string(),
        ))
        .all(conn)
        .await?;

        for row in empresas {
            let nome = row.razao_social.replace('\\', "\\\\").replace('\'', "''");
            exec(
                manager,
                &format!("INSERT INTO companies (nome) VALUES ('{nome}')"),
            )
            .await?;
            exec(
                manager,
                &format!(
                    "UPDATE empresas SET company_id = LAST_INSERT_ID() WHERE id = {}",
                    row.id
                ),
            )
            .await?;
        }

        exec(
            manager,
            "INSERT INTO companies_usuarios (user_id, company_id, papel, criado_em)
             SELECT eu.user_id, e.company_id, eu.papel, eu.criado_em
             FROM empresas_usuarios eu
             INNER JOIN empresas e ON e.id = eu.empresa_id",
        )
        .await?;

        exec(
            manager,
            "ALTER TABLE empresas
                MODIFY COLUMN company_id BIGINT NOT NULL",
        )
        .await?;

        exec(
            manager,
            "ALTER TABLE empresas
                ADD CONSTRAINT fk_empresas_company_id
                FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE CASCADE",
        )
        .await?;

        exec(
            manager,
            "ALTER TABLE empresas_usuarios DROP FOREIGN KEY fk_empresas_usuarios_user_id",
        )
        .await?;
        exec(
            manager,
            "ALTER TABLE empresas_usuarios DROP FOREIGN KEY fk_empresas_usuarios_empresa_id",
        )
        .await?;
        drop_table(manager, "empresas_usuarios").await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_table(manager, "empresas_usuarios", |t| {
            t.big_integer("user_id").not_null();
            t.big_integer("empresa_id").not_null();
            t.string("papel").not_null().default("membro");
            t.timestamp("criado_em")
                .not_null()
                .default(Expr::current_timestamp());
        })
        .await?;

        exec(
            manager,
            "INSERT INTO empresas_usuarios (user_id, empresa_id, papel, criado_em)
             SELECT cu.user_id, e.id, cu.papel, cu.criado_em
             FROM companies_usuarios cu
             INNER JOIN empresas e ON e.company_id = cu.company_id",
        )
        .await?;

        exec(
            manager,
            "ALTER TABLE empresas DROP FOREIGN KEY fk_empresas_company_id",
        )
        .await?;
        exec(manager, "ALTER TABLE empresas DROP COLUMN company_id").await?;
        drop_table(manager, "companies_usuarios").await?;
        drop_table(manager, "companies").await
    }
}
