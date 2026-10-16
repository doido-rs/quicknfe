//! Integração com a API SEFIN Nacional (NFS-e padrão nacional).

mod ambiente;
mod assinatura;
mod cliente;
mod compactacao;
mod dps;
pub mod dps_context;
mod dps_layout;
mod dps_layout_csv;
mod dps_metrics;
mod dps_presenca;
mod endereco_brasil;
mod documento_processamento;
mod dps_rn;
mod dps_rn_engine;
mod resposta;
mod rn_validacao;
mod transmissao;

pub use assinatura::assinar_dps_xml_rascunho;
pub use dps::{montar_dps_xml, montar_dps_xml_context};
pub use dps_context::{load_dps_context, DpsContext};
pub use rn_validacao::validar_dps_contexto;

pub use crate::connectors::mtls::{build_mtls_client, CredenciaisMtls};
pub use resposta::MensagemProcessamento;
pub use documento_processamento::{
    assinar_documento_nfse, certificado_para_assinatura, documento_status,
    marcar_erro_assinatura, marcar_erro_transmissao, reject_in_flight, DocumentoAssinarParams,
    DocumentoEnqueueResponse,
};
pub use transmissao::{transmitir_documento, TransmitirParams};
