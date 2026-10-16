//! Assinatura XML-DSig (enveloped) sobre `infDPS` com certificado A1 PFX.

use std::fs;

use oxixml_c14n::CanonicalizationAlgorithm;
use oxixml_dsig::{
    DigestAlgorithm, KeyInfoSpec, ReferenceSpec, RsaPrivateKey, SignatureAlgorithm,
    SignatureBuilder, SigningKey, TransformSpec, sign,
};

#[cfg(test)]
use oxixml_dsig::{Signature, TrustedCertificates, Verifier};
use oxixml_dom::{build, SerializeOptions, XmlDeclaration};

use super::xml_prolog::garantir_prolog_utf8;

use super::dps::DPS_XML_NS;
use crate::services::api_error::ApiError;
use crate::services::certificado_pfx::{parse_pfx_bytes, validar_certificado_emitente};

const DPS_XML_NS_LEGACY: &str = "http://www.gov.br/nfse";

/// Assina a DPS com certificado A1 PFX (XML-DSig enveloped sobre `infDPS`).
///
/// Quando `cpf_cnpj_emitente` é informado, confere se o CPF/CNPJ do certificado ICP-Brasil
/// corresponde ao prestador (regra E0718 / E0716 na SEFIN).
pub fn assinar_dps_xml_rascunho(
    xml_rascunho: &str,
    caminho_pfx: Option<&str>,
    senha_pfx: Option<&str>,
    cpf_cnpj_emitente: Option<&str>,
) -> Result<String, ApiError> {
    let xml_rascunho = preparar_xml_para_assinatura(xml_rascunho);

    let caminho = caminho_pfx.filter(|s| !s.trim().is_empty()).ok_or_else(|| {
        ApiError::Validation(
            "certificado digital A1 (PFX) é obrigatório para assinar a DPS".into(),
        )
    })?;
    let senha = senha_pfx.unwrap_or("");

    let pfx_bytes = fs::read(caminho).map_err(|e| {
        ApiError::Validation(format!("não foi possível ler certificado {caminho}: {e}"))
    })?;
    let parsed = parse_pfx_bytes(&pfx_bytes, senha)?;
    if let Some(insc) = cpf_cnpj_emitente {
        validar_certificado_emitente(&parsed.cert, insc)?;
    }

    assinar_com_chave(&xml_rascunho, &parsed.pkey, &parsed.cert)
}

/// Remove assinatura anterior e corrige namespace legado antes de assinar de novo.
fn preparar_xml_para_assinatura(xml: &str) -> String {
    let mut out = remover_blocos_signature(xml);
    if out.contains(DPS_XML_NS_LEGACY) {
        out = out.replace(DPS_XML_NS_LEGACY, DPS_XML_NS);
    }
    out
}

fn remover_blocos_signature(xml: &str) -> String {
    let mut out = xml.to_string();
    loop {
        let Some(start) = out.find("<Signature") else {
            break;
        };
        let Some(rel_end) = out[start..].find("</Signature>") else {
            break;
        };
        let end = start + rel_end + "</Signature>".len();
        out.replace_range(start..end, "");
    }
    out
}

fn assinar_com_chave(
    xml: &str,
    pkey: &openssl::pkey::PKey<openssl::pkey::Private>,
    cert: &openssl::x509::X509,
) -> Result<String, ApiError> {
    let id_value = extrair_id_inf_dps(xml)?;

    let pkcs8 = pkey.private_key_to_pkcs8().map_err(|e| {
        ApiError::Validation(format!("não foi possível extrair chave privada RSA do PFX: {e}"))
    })?;
    let rsa_private = RsaPrivateKey::from_der(pkcs8.as_ref()).map_err(|e| {
        ApiError::Validation(format!("chave privada RSA inválida para assinatura XML: {e}"))
    })?;
    let signing_key = SigningKey::Rsa(Box::new(rsa_private));

    let cert_der = cert
        .to_der()
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let mut document = build::parse_str(xml).map_err(|e| {
        ApiError::Validation(format!("XML da DPS inválido antes da assinatura: {e}"))
    })?;
    let dps_root = document.document_element().ok_or_else(|| {
        ApiError::Validation("XML da DPS sem elemento raiz".into())
    })?;

    // Perfil usual NFS-e Nacional: RSA-SHA1, C14N 1.0, referência ao Id de infDPS, enveloped.
    let sig_node = SignatureBuilder::new(SignatureAlgorithm::RsaSha1)
        .canonicalization(CanonicalizationAlgorithm::Canonical1_0)
        .reference(
            ReferenceSpec::new(format!("#{id_value}"))
                .digest(DigestAlgorithm::Sha1)
                .transform(TransformSpec::EnvelopedSignature)
                .transform(TransformSpec::canonicalization(
                    CanonicalizationAlgorithm::Canonical1_0,
                )),
        )
        .key_info(KeyInfoSpec::X509Certificate(cert_der.clone()))
        .build_into(&mut document, dps_root)
        .map_err(|e| ApiError::Validation(format!("falha ao montar assinatura XML: {e}")))?;

    sign(&mut document, sig_node, &signing_key).map_err(|e| {
        ApiError::Validation(format!("falha ao assinar XML da DPS: {e}"))
    })?;

    document.set_encoding(Some(super::xml_prolog::DPS_XML_ENCODING));
    let options = SerializeOptions::new().with_xml_declaration(XmlDeclaration::Include);
    Ok(garantir_prolog_utf8(&document.to_xml_with(&options)))
}

fn extrair_id_inf_dps(xml: &str) -> Result<String, ApiError> {
    for tag in ["infDPS", "infDps"] {
        if let Some(start) = xml.find(&format!("<{tag}")) {
            return extract_id_attr(&xml[start..]);
        }
    }
    Err(ApiError::Validation("XML sem infDPS para assinar".into()))
}

fn extract_id_attr(fragment: &str) -> Result<String, ApiError> {
    for key in [r#"Id=""#, r#"ID=""#] {
        if let Some(rel) = fragment.find(key) {
            let rest = &fragment[rel + key.len()..];
            if let Some(end) = rest.find('"') {
                return Ok(rest[..end].to_string());
            }
        }
    }
    Err(ApiError::Validation("infDPS sem atributo Id".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::certificado_pfx::parse_pfx_bytes;
    use crate::services::openssl_legacy::ensure_legacy_providers_for_pkcs12;
    use std::path::PathBuf;

    #[test]
    fn rejeita_sem_certificado() {
        let xml = r#"<DPS><infDPS Id="DPS1"><tpAmb>2</tpAmb></infDPS></DPS>"#;
        assert!(assinar_dps_xml_rascunho(xml, None, None, None).is_err());
    }

    #[test]
    fn re_assina_substituindo_signature_e_namespace_legado() {
        let pfx = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/test-sign.pfx");
        let xml = r#"<?xml version="1.0"?><DPS xmlns="http://www.gov.br/nfse"><infDPS Id="DPS1"><tpAmb>2</tpAmb></infDPS><Signature xmlns="http://www.w3.org/2000/09/xmldsig#"><SignatureValue>old</SignatureValue></Signature></DPS>"#;
        let out = assinar_dps_xml_rascunho(
            xml,
            Some(pfx.to_str().unwrap()),
            Some("test123"),
            None,
        )
        .unwrap();
        assert!(out.contains(DPS_XML_NS));
        assert!(!out.contains(DPS_XML_NS_LEGACY));
        assert!(out.contains("<SignatureValue>"));
        assert!(!out.contains(">old<"));
    }

    #[test]
    fn assina_com_pfx_fixture_e_verifica_localmente() {
        let pfx = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/test-sign.pfx");
        let pfx_bytes = std::fs::read(&pfx).unwrap();
        ensure_legacy_providers_for_pkcs12();
        let parsed = parse_pfx_bytes(&pfx_bytes, "test123").unwrap();
        let cert_der = parsed.cert.to_der().unwrap();

        let xml = r#"<?xml version="1.0"?><DPS xmlns="http://www.sped.fazenda.gov.br/nfse"><infDPS Id="DPS1"><tpAmb>2</tpAmb></infDPS></DPS>"#;
        let out = assinar_dps_xml_rascunho(
            xml,
            Some(pfx.to_str().unwrap()),
            Some("test123"),
            None,
        )
        .unwrap();
        assert!(out.contains("<SignatureValue>"));
        assert!(out.contains("<X509Certificate>"));
        assert!(out.contains("rsa-sha1"));
        assert!(
            out.contains(r#"encoding="UTF-8""#),
            "SEFIN E1229 exige declaração UTF-8"
        );

        let document = build::parse_str(&out).expect("parse signed");
        let signature = Signature::find(&document).expect("signature");
        let trusted = TrustedCertificates::new().with_certificate(cert_der);
        let result = Verifier::new()
            .verify(&document, &signature, &trusted)
            .expect("verify");
        assert!(result.core_valid(), "assinatura local deve ser válida");
    }
}
