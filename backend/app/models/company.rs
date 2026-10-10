//! Model extensions for Company (tenant).
#![allow(dead_code, unused_imports)]

pub use super::_entities::companies::*;

use doido::model::sea_orm::ActiveModelBehavior;

impl ActiveModelBehavior for ActiveModel {}
