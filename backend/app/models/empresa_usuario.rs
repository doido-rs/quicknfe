//! Vínculo multi-tenant usuário ↔ empresa.
#![allow(dead_code, unused_imports)]

pub use super::_entities::empresas_usuarios::*;

use doido::model::sea_orm::ActiveModelBehavior;

impl ActiveModelBehavior for ActiveModel {}
