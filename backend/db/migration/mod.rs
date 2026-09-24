pub use doido::model::sea_orm_migration::prelude::*;

mod m20260101000000_create_storage_tables;
mod m20260101000001_create_doido_jobs_table;
mod m20260924120000_create_documentos_fiscais_schema;
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
            // @generated-migrations-list
        ]
    }
}
