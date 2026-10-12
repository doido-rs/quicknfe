//! Model extensions for CNA (código de tributação nacional).

pub use super::_entities::cnas::*;

use doido::model::sea_orm::ActiveModelBehavior;

impl ActiveModelBehavior for ActiveModel {}
