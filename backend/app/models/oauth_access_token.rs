//! Model extensions for `oauth_access_token` — safe to edit; never overwritten by generators.
//!
//! The SeaORM entity definition lives in `_entities/oauth_access_tokens.rs` and is
//! regenerated on every `doido db migrate`.
#![allow(dead_code, unused_imports)]

pub use super::_entities::oauth_access_tokens::*;

use doido::model::sea_orm::ActiveModelBehavior;

impl ActiveModelBehavior for ActiveModel {}
