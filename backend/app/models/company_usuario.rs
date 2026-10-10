//! Membership between users and tenant companies.
#![allow(dead_code, unused_imports)]

pub use super::_entities::companies_usuarios::*;

use doido::model::sea_orm::ActiveModelBehavior;

impl ActiveModelBehavior for ActiveModel {}
