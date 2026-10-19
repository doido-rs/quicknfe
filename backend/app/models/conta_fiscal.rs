//! Model extensions for ContaFiscal.
#![allow(dead_code, unused_imports)]

pub use super::_entities::contas_fiscais::*;

use doido::model::sea_orm::ActiveModelBehavior;

impl ActiveModelBehavior for ActiveModel {}
