//! Model extensions for `oauth_authorization_code` — safe to edit; never overwritten by generators.
//!
//! The SeaORM entity definition lives in `_entities/oauth_authorization_codes.rs` and is
//! regenerated on every `doido db migrate`.
#![allow(dead_code, unused_imports)]

pub use super::_entities::oauth_authorization_codes::*;

use doido::model::sea_orm::ActiveModelBehavior;

impl ActiveModelBehavior for ActiveModel {}
