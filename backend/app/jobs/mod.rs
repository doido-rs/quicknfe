//! Background jobs (doido-jobs). Each `doido generate job <Name>` registers its
//! module here, just above the marker below.

pub mod import_nfse_anexo_a;
pub mod sync_nfse_catalog_daily;
pub mod assinar_documento;
pub mod transmitir_documento;

// @generated-jobs
