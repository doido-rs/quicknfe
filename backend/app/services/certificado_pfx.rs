//! Verificação de PKCS#12 (.pfx/.p12) e extração de validade / inscrição ICP-Brasil.

use chrono::{DateTime, NaiveDateTime, Utc};
use foreign_types::ForeignTypeRef;
use openssl_sys as ffi;
use openssl::nid::Nid;
use openssl::pkcs12::Pkcs12;
use openssl::pkey::Private;
use openssl::pkey::PKey;
use openssl::x509::X509;

use crate::services::api_error::ApiError;
use crate::services::openssl_legacy::ensure_legacy_providers_for_pkcs12;

/// Chave privada + certificado extraídos de um PFX verificado.
pub struct ParsedPfx {
    pub pkey: PKey<Private>,
    pub cert: X509,
}

/// Abre o PFX com a senha informada e retorna chave + certificado.
pub fn parse_pfx_bytes(bytes: &[u8], senha: &str) -> Result<ParsedPfx, ApiError> {
    if bytes.is_empty() {
        return Err(ApiError::Validation(
            "arquivo do certificado está vazio".into(),
        ));
    }
    ensure_legacy_providers_for_pkcs12();
    let pkcs12 = Pkcs12::from_der(bytes)
        .and_then(|p| p.parse2(senha))
        .map_err(pfx_parse_error)?;
    let pkey = pkcs12
        .pkey
        .ok_or_else(|| ApiError::Validation("PFX sem chave privada".into()))?;
    let cert = pkcs12
        .cert
        .ok_or_else(|| ApiError::Validation("PFX sem certificado".into()))?;
    Ok(ParsedPfx { pkey, cert })
}

/// Abre o PFX com a senha informada e retorna a validade (notAfter) do certificado.
pub fn verificar_pfx(bytes: &[u8], senha: &str) -> Result<DateTime<Utc>, ApiError> {
    let parsed = parse_pfx_bytes(bytes, senha)?;
    data_validade_from_cert(&parsed.cert)
}

/// CPF/CNPJ do titular quando presente no certificado ICP-Brasil (OtherName ou subject).
pub fn inscricao_federal_do_certificado(cert: &openssl::x509::X509Ref) -> Option<String> {
    if let Some(s) = inscricao_de_subject(cert) {
        return Some(s);
    }
    let candidatos = inscricoes_icp_san(cert);
    candidatos
        .iter()
        .find(|c| c.len() == 14)
        .or_else(|| candidatos.first())
        .cloned()
}

/// Garante que o certificado pertence ao emitente da DPS (evita E0718 na SEFIN).
pub fn validar_certificado_emitente(
    cert: &openssl::x509::X509Ref,
    cpf_cnpj_prestador: &str,
) -> Result<(), ApiError> {
    let esperado = apenas_digitos(cpf_cnpj_prestador);
    if esperado.len() != 11 && esperado.len() != 14 {
        return Err(ApiError::Validation(
            "CPF/CNPJ do prestador inválido para conferir o certificado.".into(),
        ));
    }
    let mut candidatos = Vec::new();
    if let Some(s) = inscricao_de_subject(cert) {
        candidatos.push(s);
    }
    candidatos.extend(inscricoes_icp_san(cert));
    candidatos.sort();
    candidatos.dedup();

    if candidatos.is_empty() {
        return Err(ApiError::Validation(
            "O certificado não possui CPF/CNPJ ICP-Brasil reconhecível (SubjectAlternativeName / subject). \
             A SEFIN rejeita com E0716/E0715 — use certificado e-CNPJ ou e-CPF do emitente."
                .into(),
        ));
    }
    if candidatos.iter().any(|c| c == &esperado) {
        return Ok(());
    }
    let lista = candidatos.join(", ");
    Err(ApiError::Validation(format!(
        "Nenhum CPF/CNPJ ICP do certificado ({lista}) corresponde ao prestador da nota ({esperado}). \
         Confira se o PFX é do emitente e se o cadastro do prestador usa o mesmo CNPJ/CPF (E0718)."
    )))
}

fn pfx_parse_error(e: openssl::error::ErrorStack) -> ApiError {
    let msg = e
        .errors()
        .iter()
        .map(|err| err.reason().unwrap_or("erro OpenSSL"))
        .collect::<Vec<_>>()
        .join("; ");
    let lower = msg.to_ascii_lowercase();
    if lower.contains("unsupported") || lower.contains("unknown cipher") || lower.contains("rc2") {
        return ApiError::Validation(
            "arquivo PFX usa criptografia legada (RC2/3DES); instale o módulo OpenSSL legacy \
             (ossl-modules) no servidor ou reexporte o certificado em formato AES"
                .into(),
        );
    }
    if lower.contains("mac verify failure") || lower.contains("invalid password") {
        return ApiError::Validation("senha do certificado incorreta".into());
    }
    ApiError::Validation(format!(
        "senha incorreta ou arquivo de certificado inválido ({msg})"
    ))
}

fn data_validade_from_cert(cert: &openssl::x509::X509Ref) -> Result<DateTime<Utc>, ApiError> {
    let s = cert.not_after().to_string();
    let ndt = NaiveDateTime::parse_from_str(s.trim(), "%b %d %H:%M:%S %Y GMT").map_err(|e| {
        ApiError::Internal(format!("validade do certificado (notAfter): {e}"))
    })?;
    Ok(DateTime::from_naive_utc_and_offset(ndt, Utc))
}

fn apenas_digitos(s: &str) -> String {
    s.chars().filter(|c| c.is_ascii_digit()).collect()
}

fn inscricao_de_subject(cert: &openssl::x509::X509Ref) -> Option<String> {
    for nid in [Nid::SERIALNUMBER, Nid::COMMONNAME] {
        for entry in cert.subject_name().entries_by_nid(nid) {
            if let Ok(raw) = entry.data().to_string() {
                if let Some(d) = extrair_inscricao_de_texto(&raw) {
                    return Some(d);
                }
            }
        }
    }
    None
}

/// Valor DER da extensão SubjectAlternativeName (GeneralNames), não o certificado inteiro.
fn subject_alt_name_ext_value(cert: &openssl::x509::X509Ref) -> Option<Vec<u8>> {
    unsafe {
        ffi::init();
        let idx = ffi::X509_get_ext_by_NID(
            cert.as_ptr(),
            Nid::SUBJECT_ALT_NAME.as_raw(),
            -1,
        );
        if idx < 0 {
            return None;
        }
        let ext = ffi::X509_get_ext(cert.as_ptr(), idx);
        if ext.is_null() {
            return None;
        }
        let asn1 = ffi::X509_EXTENSION_get_data(ext);
        if asn1.is_null() {
            return None;
        }
        let asn1_str = asn1 as *mut ffi::ASN1_STRING;
        let len = ffi::ASN1_STRING_length(asn1_str);
        if len <= 0 {
            return None;
        }
        let ptr = ffi::ASN1_STRING_get0_data(asn1_str);
        Some(std::slice::from_raw_parts(ptr as *const u8, len as usize).to_vec())
    }
}

fn inscricoes_icp_san(cert: &openssl::x509::X509Ref) -> Vec<String> {
    subject_alt_name_ext_value(cert)
        .map(|san| inscricoes_de_san_der(&san))
        .unwrap_or_default()
}

fn inscricoes_de_san_der(der: &[u8]) -> Vec<String> {
    const OID_CNPJ: [u8; 8] = [0x06, 0x08, 0x60, 0x4C, 0x01, 0x03, 0x03, 0x03];
    const OID_CPF: [u8; 8] = [0x06, 0x08, 0x60, 0x4C, 0x01, 0x03, 0x03, 0x01];
    const OID_CNPJ_BODY: [u8; 6] = [0x60, 0x4C, 0x01, 0x03, 0x03, 0x03];
    const OID_CPF_BODY: [u8; 6] = [0x60, 0x4C, 0x01, 0x03, 0x03, 0x01];

    let mut found = Vec::new();
    for oid in [&OID_CNPJ[..], &OID_CPF[..], &OID_CNPJ_BODY[..], &OID_CPF_BODY[..]] {
        let mut from = 0;
        while from < der.len() {
            let Some(rel) = der[from..].windows(oid.len()).position(|w| w == oid) else {
                break;
            };
            let pos = from + rel;
            if let Some(insc) = inscricao_apos_oid(&der[pos + oid.len()..]) {
                found.push(insc);
            }
            from = pos + oid.len().max(1);
        }
    }
    found.sort();
    found.dedup();
    found
}

fn inscricao_apos_oid(der: &[u8]) -> Option<String> {
    inscricao_ascii_em_der(der, 14).or_else(|| inscricao_ascii_em_der(der, 11))
}

fn inscricao_ascii_em_der(der: &[u8], len: usize) -> Option<String> {
    if der.len() < len {
        return None;
    }
    for i in 0..=der.len() - len {
        let slice = &der[i..i + len];
        if slice.iter().all(|b| b.is_ascii_digit()) {
            return Some(String::from_utf8(slice.to_vec()).ok()?);
        }
    }
    None
}

fn extrair_inscricao_de_texto(text: &str) -> Option<String> {
    if let Some(after_colon) = text.rsplit(':').next() {
        let d = apenas_digitos(after_colon);
        if d.len() == 14 || d.len() == 11 {
            return Some(d);
        }
    }
    let d = apenas_digitos(text);
    if d.len() == 14 || d.len() == 11 {
        return Some(d);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn verificar_pfx_fixture_ok() {
        let pfx = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/test-sign.pfx");
        let bytes = fs::read(&pfx).expect("read pfx");
        let validade = verificar_pfx(&bytes, "test123").expect("verify");
        assert!(validade > Utc::now());
    }

    #[test]
    fn verificar_pfx_senha_errada() {
        let pfx = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/test-sign.pfx");
        let bytes = fs::read(&pfx).expect("read pfx");
        assert!(verificar_pfx(&bytes, "wrong").is_err());
    }

    #[test]
    fn fixture_test_sign_nao_tem_inscricao_icp() {
        let pfx = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/test-sign.pfx");
        let bytes = fs::read(&pfx).expect("read pfx");
        let parsed = parse_pfx_bytes(&bytes, "test123").expect("parse");
        assert!(inscricao_federal_do_certificado(&parsed.cert).is_none());
        assert!(validar_certificado_emitente(&parsed.cert, "12345678000195").is_err());
    }

    #[test]
    fn extrair_inscricao_de_texto_colon() {
        assert_eq!(
            extrair_inscricao_de_texto("EMPRESA LTDA:12345678000195"),
            Some("12345678000195".into())
        );
    }

    #[test]
    fn san_icp_lista_todos_cnpj_othername() {
        const OID_CNPJ: [u8; 8] = [0x06, 0x08, 0x60, 0x4C, 0x01, 0x03, 0x03, 0x03];
        let mut san = Vec::new();
        san.extend_from_slice(&OID_CNPJ);
        san.extend_from_slice(b"xx24626357000148yy");
        san.extend_from_slice(&OID_CNPJ);
        san.extend_from_slice(b"zz31041140000131");
        let all = inscricoes_de_san_der(&san);
        assert!(all.contains(&"24626357000148".into()));
        assert!(all.contains(&"31041140000131".into()));
    }

    /// PFX ICP-Brasil com RC2 (OpenSSL 3). Arquivo + senha via env (não commitar senha).
    #[test]
    fn verificar_pfx_legacy_rc2_env_fixture() {
        let Some(pwd) = std::env::var("QUICKNFE_LEGACY_PFX_PASSWORD").ok() else {
            return;
        };
        let pfx = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../certificado.pfx");
        if !pfx.exists() {
            return;
        }
        let bytes = fs::read(&pfx).expect("read pfx");
        let validade = verificar_pfx(&bytes, &pwd).expect("legacy rc2 pfx");
        assert!(validade > Utc::now());
        let parsed = parse_pfx_bytes(&bytes, &pwd).expect("parse");
        if let Some(insc) = inscricao_federal_do_certificado(&parsed.cert) {
            assert!(insc.len() == 11 || insc.len() == 14);
        }
    }
}
