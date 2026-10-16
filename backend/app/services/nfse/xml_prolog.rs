//! Prolog XML exigido na recepção SEFIN (regra E1229 — UTF-8).

pub const DPS_XML_ENCODING: &str = "UTF-8";

/// Garante `<?xml version="1.0" encoding="UTF-8"?>` antes do documento.
pub fn garantir_prolog_utf8(xml: &str) -> String {
    let body = xml.trim_start();
    if !body.starts_with("<?xml") {
        return format!(r#"<?xml version="1.0" encoding="{DPS_XML_ENCODING}"?>{body}"#);
    }
    let Some(end) = body.find("?>") else {
        return body.to_string();
    };
    let decl = &body[..end];
    if decl.contains("encoding=") {
        return body.to_string();
    }
    format!(
        r#"{} encoding="{DPS_XML_ENCODING}"?>{}"#,
        decl,
        &body[end + 2..]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adiciona_prolog_quando_ausente() {
        let out = garantir_prolog_utf8("<DPS/>");
        assert!(out.starts_with(r#"<?xml version="1.0" encoding="UTF-8"?>"#));
    }

    #[test]
    fn adiciona_encoding_quando_decl_sem_encoding() {
        let out = garantir_prolog_utf8(r#"<?xml version="1.0"?><DPS/>"#);
        assert!(out.starts_with(r#"<?xml version="1.0" encoding="UTF-8"?>"#));
    }

    #[test]
    fn preserva_quando_ja_tem_utf8() {
        let raw = r#"<?xml version="1.0" encoding="UTF-8"?><DPS/>"#;
        assert_eq!(garantir_prolog_utf8(raw), raw);
    }
}
