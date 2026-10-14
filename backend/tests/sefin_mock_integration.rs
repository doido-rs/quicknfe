//! Transmissão contra `sefin-mock` (manual/CI opcional).
//!
//! Requer: `make dev-sefin-mock`, MySQL dev, `SEFIN_BASE_URL=http://127.0.0.1:8090`.
//! Rode com: `cargo test --test sefin_mock_integration -- --ignored`

#[test]
#[ignore = "requer sefin-mock + banco dev configurados"]
fn transmissao_documento_assinado_no_mock() {
    // Placeholder de integração: expandir com cliente HTTP autenticado quando CI tiver stack completa.
    let _ = std::env::var("SEFIN_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:8090".into());
}
