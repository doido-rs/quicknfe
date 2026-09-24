//! Model extensions for EventoDocumentoFiscal.
#![allow(dead_code, unused_imports)]

pub use super::_entities::eventos_documento_fiscal::*;

use doido::model::sea_orm::ActiveModelBehavior;

impl ActiveModelBehavior for ActiveModel {}
