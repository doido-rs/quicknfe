//! Model extensions for CertificadoDigital.
#![allow(dead_code, unused_imports)]

pub use super::_entities::certificados_digitais::*;

use doido::model::sea_orm::ActiveModelBehavior;

impl ActiveModelBehavior for ActiveModel {}
