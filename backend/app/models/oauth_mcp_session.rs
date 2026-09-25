//! Model extensions for `oauth_mcp_session` — safe to edit; never overwritten by generators.
//!
//! The SeaORM entity definition lives in `_entities/oauth_mcp_sessions.rs` and is
//! regenerated on every `doido db migrate`.
#![allow(dead_code, unused_imports)]

pub use super::_entities::oauth_mcp_sessions::*;

use doido::model::sea_orm::ActiveModelBehavior;

impl ActiveModelBehavior for ActiveModel {}
