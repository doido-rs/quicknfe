//! Model extensions for Empresa — safe to edit; never overwritten by generators.
//!
//! The SeaORM entity definition lives in `_entities/empresas.rs` and is
//! regenerated on every `doido db migrate`.
#![allow(dead_code, unused_imports)]

pub use super::_entities::empresas::*;

use doido::model::sea_orm::ActiveModelBehavior;

impl ActiveModelBehavior for ActiveModel {}
