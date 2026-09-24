//! Model extensions for DocumentoFiscal.
#![allow(dead_code, unused_imports)]

pub use super::_entities::documentos_fiscais::*;

use doido::model::sea_orm::ActiveModelBehavior;

impl ActiveModelBehavior for ActiveModel {}
