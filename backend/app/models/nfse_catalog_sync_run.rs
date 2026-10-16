//! Histórico de execuções de sync do catálogo NFS-e.

pub use super::_entities::nfse_catalog_sync_runs::*;

use doido::model::sea_orm::ActiveModelBehavior;

impl ActiveModelBehavior for ActiveModel {}
