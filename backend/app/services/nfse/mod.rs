//! Integração com a API SEFIN Nacional (NFS-e padrão nacional).

mod ambiente;
mod cliente;
mod compactacao;
mod resposta;
mod transmissao;

pub use resposta::MensagemProcessamento;
pub use transmissao::{transmitir_documento, TransmitirParams};
