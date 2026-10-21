//! Integração com a API SEFIN Nacional (NFS-e padrão nacional).

mod ambiente;
mod assinatura;
pub mod catalog_daily;
mod catalog_sync_runs;
mod catalog_sync_schedule;
mod codigo_servico;
mod e0312_validacao;
mod incidencia_issqn;
mod municipio_cna_sync;
mod municipio_cnas;
mod nfse_catalogo_nacional_import;
mod nfse_incidencia_import;
mod cliente;
mod compactacao;
mod dps;
pub mod dps_context;
mod dps_layout;
mod dps_layout_csv;
#[cfg(test)]
mod dps_metrics;
mod dps_presenca;
mod endereco_brasil;
mod documento_processamento;
mod dps_rn;
mod dps_rn_engine;
mod resposta;
mod rn_validacao;
mod transmissao;
mod xml_prolog;

pub use codigo_servico::normalize_c_trib_nac;
pub use assinatura::assinar_dps_xml_rascunho;
pub use dps::montar_dps_xml_context;
pub use dps_context::load_dps_context;
pub use catalog_sync_schedule::next_daily_catalog_run;
pub use municipio_cna_sync::{municipio_tem_parametrizacao, sync_municipios_batch};
pub use municipio_cnas::list_cnas_administrados;
pub use nfse_catalogo_nacional_import::run as import_catalogo_nacional;
pub use nfse_incidencia_import::run as import_incidencia_cna;
pub use rn_validacao::validar_dps_contexto_com_catalogo;

pub use crate::connectors::mtls::CredenciaisMtls;
pub use resposta::MensagemProcessamento;
pub use documento_processamento::{
    assinar_documento_nfse, marcar_erro_assinatura, marcar_erro_transmissao, reject_in_flight,
    DocumentoAssinarParams, DocumentoEnqueueResponse,
};
pub use transmissao::{transmitir_documento, TransmitirParams};
