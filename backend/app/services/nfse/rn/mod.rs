//! Validação RN DPS — catálogo, grupos e execução no save.

pub mod dispatch_generated;
pub mod groups;
pub mod runner;
pub mod violations;

#[cfg(test)]
mod catalog_tests;
#[cfg(test)]
mod cases_tests;

pub use runner::{validar_rn_com_catalogo, validar_rn_sync};
