//! Integração com a API SEFIN Nacional (NFS-e padrão nacional).

mod ambiente;
mod cliente;
mod compactacao;
mod resposta;
mod transmissao;

pub use cliente::{build_mtls_client, CredenciaisMtls};
pub use resposta::MensagemProcessamento;
pub use transmissao::{transmitir_documento, TransmitirParams};
