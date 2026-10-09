//! Model extensions for Intermediario.
#![allow(dead_code, unused_imports)]

pub use super::_entities::intermediarios::*;

use doido::model::sea_orm::ActiveModelBehavior;

impl ActiveModelBehavior for ActiveModel {}
