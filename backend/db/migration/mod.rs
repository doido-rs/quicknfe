pub use doido::model::sea_orm_migration::prelude::*;

mod m20260101000000_create_storage_tables;
mod m20260101000001_create_doido_jobs_table;
mod m20260924120000_create_documentos_fiscais_schema;
mod m20260924_202000_create_users_table;
mod m20260924121500_create_empresas_usuarios_table;
mod m20260924_202217_create_empresas_usuarios_table;
mod m20260925_120000_create_oauth_mcp_tables;
mod m20261009180000_create_intermediarios;
mod m20261009210000_companies_as_tenant;
mod m20261010180000_create_municipios;
mod m20261011140000_regime_tributario_enum;
mod m20261011180000_documento_serie_numero_automatico;
mod m20261011210000_documento_partes_localidade;
mod m20261011230000_documento_municipios;
mod m20261012010000_create_paises;
mod m20261012120000_create_cnas;
mod m20261012140000_item_descricao_servico;
// @generated-migrations-mod — `doido generate model` inserts `mod` declarations above this line. Do not remove.

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        // Register migrations here, oldest first. `doido generate model` inserts
        // entries above the marker below. Do not remove the marker.
        vec![
            Box::new(m20260101000000_create_storage_tables::Migration),
            Box::new(m20260101000001_create_doido_jobs_table::Migration),
            Box::new(m20260924120000_create_documentos_fiscais_schema::Migration),
            Box::new(m20260924_202000_create_users_table::Migration),
            Box::new(m20260924121500_create_empresas_usuarios_table::Migration),
            Box::new(m20260924_202217_create_empresas_usuarios_table::Migration),
            Box::new(m20260925_120000_create_oauth_mcp_tables::Migration),
            Box::new(m20261009180000_create_intermediarios::Migration),
            Box::new(m20261009210000_companies_as_tenant::Migration),
            Box::new(m20261010180000_create_municipios::Migration),
            Box::new(m20261011140000_regime_tributario_enum::Migration),
            Box::new(m20261011180000_documento_serie_numero_automatico::Migration),
            Box::new(m20261011210000_documento_partes_localidade::Migration),
            Box::new(m20261011230000_documento_municipios::Migration),
            Box::new(m20261012010000_create_paises::Migration),
            Box::new(m20261012120000_create_cnas::Migration),
            Box::new(m20261012140000_item_descricao_servico::Migration),
            // @generated-migrations-list
        ]
    }
}
