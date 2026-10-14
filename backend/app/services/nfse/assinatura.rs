//! Assinatura XML-DSig (enveloped) sobre `infDPS` com certificado A1 PFX.

use std::fs;

use base64::{engine::general_purpose::STANDARD, Engine as _};
use openssl::hash::MessageDigest;
use openssl::pkcs12::Pkcs12;
use openssl::pkey::PKey;
use openssl::sign::Signer;
use openssl::x509::X509;
use sha2::{Digest, Sha256};

use crate::services::api_error::ApiError;

/// Assina a DPS com certificado A1 PFX (XML-DSig enveloped sobre `infDPS`).
pub fn assinar_dps_xml_rascunho(
    xml_rascunho: &str,
    caminho_pfx: Option<&str>,
    senha_pfx: Option<&str>,
) -> Result<String, ApiError> {
    if xml_rascunho.contains("<Signature") || xml_rascunho.contains(":Signature") {
        return Ok(xml_rascunho.to_string());
    }

    let caminho = caminho_pfx.filter(|s| !s.trim().is_empty()).ok_or_else(|| {
        ApiError::Validation(
            "certificado digital A1 (PFX) é obrigatório para assinar a DPS".into(),
        )
    })?;
    let senha = senha_pfx.unwrap_or("");

    let pfx_bytes = fs::read(caminho).map_err(|e| {
        ApiError::Validation(format!("não foi possível ler certificado {caminho}: {e}"))
    })?;
    let pkcs12 = Pkcs12::from_der(&pfx_bytes)
        .and_then(|p| p.parse2(senha))
        .map_err(|e| ApiError::Validation(format!("PFX inválido ou senha incorreta: {e}")))?;
    let pkey = pkcs12.pkey.ok_or_else(|| ApiError::Validation("PFX sem chave privada".into()))?;
    let cert = pkcs12
        .cert
        .ok_or_else(|| ApiError::Validation("PFX sem certificado".into()))?;

    assinar_com_chave(xml_rascunho, &pkey, &cert)
}

fn assinar_com_chave(xml: &str, pkey: &PKey<openssl::pkey::Private>, cert: &X509) -> Result<String, ApiError> {
    let (inf_start, inf_end, id_value) = locate_inf_dps(xml)?;
    let inf_fragment = &xml[inf_start..inf_end];
    let digest_b64 = base64_sha256(inf_fragment.as_bytes());

    let cert_der = cert.to_der().map_err(|e| ApiError::Internal(e.to_string()))?;
    let cert_b64 = STANDARD.encode(&cert_der);

    let signed_info = format!(
        r##"<SignedInfo xmlns="http://www.w3.org/2000/09/xmldsig#">
  <CanonicalizationMethod Algorithm="http://www.w3.org/TR/2001/REC-xml-c14n-20010315"/>
  <SignatureMethod Algorithm="http://www.w3.org/2001/04/xdgst-more#rsa-sha256"/>
  <Reference URI="#{id_value}">
    <Transforms>
      <Transform Algorithm="http://www.w3.org/2000/09/xmldsig#enveloped-signature"/>
    </Transforms>
    <DigestMethod Algorithm="http://www.w3.org/2001/04/xmlenc#sha256"/>
    <DigestValue>{digest_b64}</DigestValue>
  </Reference>
</SignedInfo>"##
    );

    let mut signer = Signer::new(MessageDigest::sha256(), pkey)
        .map_err(|e| ApiError::Internal(format!("signer: {e}")))?;
    signer
        .update(signed_info.as_bytes())
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    let sig_bytes = signer
        .sign_to_vec()
        .map_err(|e| ApiError::Internal(format!("assinatura: {e}")))?;
    let signature_value = STANDARD.encode(sig_bytes);

    let signature_block = format!(
        r##"  <Signature xmlns="http://www.w3.org/2000/09/xmldsig#">
{signed_info}
  <SignatureValue>{signature_value}</SignatureValue>
  <KeyInfo>
    <X509Data>
      <X509Certificate>{cert_b64}</X509Certificate>
    </X509Data>
  </KeyInfo>
</Signature>
"##
    );

    if xml.contains("</DPS>") {
        Ok(xml.replace("</DPS>", &format!("{signature_block}</DPS>")))
    } else {
        Ok(format!("{xml}\n{signature_block}"))
    }
}

fn locate_inf_dps(xml: &str) -> Result<(usize, usize, String), ApiError> {
    for tag in ["infDPS", "infDps"] {
        if let Some(start) = xml.find(&format!("<{tag}")) {
            let id_value = extract_id_attr(&xml[start..])?;
            let open_end = xml[start..]
                .find('>')
                .map(|i| start + i + 1)
                .ok_or_else(|| ApiError::Validation("infDPS malformado".into()))?;
            let close = xml[open_end..]
                .find(&format!("</{tag}>"))
                .map(|i| open_end + i)
                .ok_or_else(|| ApiError::Validation("infDPS sem fechamento".into()))?;
            return Ok((open_end, close, id_value));
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

fn base64_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    STANDARD.encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn rejeita_sem_certificado() {
        let xml = r#"<DPS><infDPS Id="DPS1"><tpAmb>2</tpAmb></infDPS></DPS>"#;
        assert!(assinar_dps_xml_rascunho(xml, None, None).is_err());
    }

    #[test]
    fn assina_com_pfx_fixture() {
        let pfx = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/test-sign.pfx");
        let xml = r#"<?xml version="1.0"?><DPS xmlns="http://www.gov.br/nfse"><infDPS Id="DPS1"><tpAmb>2</tpAmb></infDPS></DPS>"#;
        let out = assinar_dps_xml_rascunho(
            xml,
            Some(pfx.to_str().unwrap()),
            Some("test123"),
        )
        .unwrap();
        assert!(out.contains("<SignatureValue>"));
        assert!(out.contains("<X509Certificate>"));
    }
}
