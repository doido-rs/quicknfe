//! Application models. Each `doido generate model <Name>` registers its module
//! here, just above the marker below.
//!
//! Entity structs live under `_entities/` (regenerated on migrate). Edit the
//! sibling `app/models/<name>.rs` files for validations and custom behaviour.

pub mod _entities;

pub mod cna;
pub mod certificado_digital;
pub mod cliente;
pub mod company;
pub mod company_usuario;
pub mod conta_fiscal;
pub mod documento_fiscal;
pub mod empresa;
pub mod evento_documento_fiscal;
pub mod intermediario;
pub mod item_documento_fiscal;
pub mod log_mensagem_retorno;
pub mod municipio;
pub mod municipio_cna_parametrizacao;
pub mod nfse_catalog_sync_run;
pub mod pais;
pub mod lote_transmissao;
pub mod tributo_item;
pub mod storage_attachment;
pub mod storage_blob;
pub mod storage_variant_record;
pub mod user;
pub mod oauth_access_token;
pub mod oauth_authorization_code;
pub mod oauth_client;
pub mod oauth_mcp_session;
pub mod regime_tributario;
// @generated-models
