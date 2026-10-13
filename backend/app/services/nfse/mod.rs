//! Integração com a API SEFIN Nacional (NFS-e padrão nacional).

mod ambiente;
mod cliente;
mod compactacao;
mod dps;
mod resposta;
mod transmissao;

pub use dps::{assinar_dps_xml_rascunho, montar_dps_xml};

pub use cliente::{build_mtls_client, CredenciaisMtls};
pub use resposta::MensagemProcessamento;
pub use transmissao::{transmitir_documento, TransmitirParams};
