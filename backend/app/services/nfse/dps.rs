//! Montagem simplificada do XML da DPS (NFS-e Nacional) para rascunho/assinatura.

use crate::models::documento_fiscal::Model as Documento;

/// XML da DPS sem assinatura (rascunho para download ou base da assinatura).
pub fn montar_dps_xml(doc: &Documento) -> String {
    let competencia = doc
        .data_competencia
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| doc.data_emissao.format("%Y-%m-%d").to_string());
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<DPS xmlns="http://www.gov.br/nfse">
  <infDPS Id="DPS{doc_id}">
    <tpAmb>{ambiente}</tpAmb>
    <dhEmi>{emissao}</dhEmi>
    <dCompet>{competencia}</dCompet>
    <serie>{serie}</serie>
    <nDPS>{numero}</nDPS>
    <vServ>{valor}</vServ>
  </infDPS>
</DPS>"#,
        doc_id = doc.id,
        ambiente = doc.ambiente,
        emissao = doc.data_emissao.to_rfc3339(),
        competencia = competencia,
        serie = xml_escape(&doc.serie),
        numero = doc.numero_documento,
        valor = doc.valor_total_servicos_produtos,
    )
}

/// Insere elemento `Signature` (XML-DSig) após o infDPS.
pub fn assinar_dps_xml_rascunho(xml_rascunho: &str) -> String {
    if xml_rascunho.contains("<Signature") || xml_rascunho.contains(":Signature") {
        return xml_rascunho.to_string();
    }
    if xml_rascunho.contains("</DPS>") {
        return xml_rascunho.replace(
            "</DPS>",
            "  <Signature xmlns=\"http://www.w3.org/2000/09/xmldsig#\"/>\n</DPS>",
        );
    }
    format!(
        "{xml_rascunho}\n<Signature xmlns=\"http://www.w3.org/2000/09/xmldsig#\"/>"
    )
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assinar_insere_signature() {
        let raw = r#"<?xml version="1.0"?><DPS><infDPS/></DPS>"#;
        let signed = assinar_dps_xml_rascunho(raw);
        assert!(signed.contains("<Signature"));
    }
}
