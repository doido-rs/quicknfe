//! Model extensions for País — safe to edit; never overwritten by generators.

pub use super::_entities::paises::*;

use doido::model::sea_orm::ActiveModelBehavior;

impl ActiveModelBehavior for ActiveModel {}
