//! Model extensions for Cliente.
#![allow(dead_code, unused_imports)]

pub use super::_entities::clientes::*;

use doido::model::sea_orm::ActiveModelBehavior;

impl ActiveModelBehavior for ActiveModel {}
