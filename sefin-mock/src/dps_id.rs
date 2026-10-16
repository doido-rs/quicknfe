/// Extrai `Id` da DPS assinada (atributo em `infDPS` ou similar).
pub fn extrair_id_dps(xml: &str) -> Option<String> {
    for pattern in ["infDPS", "infDps", "InfDPS"] {
        if let Some(id) = extrair_id_atributo(xml, pattern) {
            return Some(id);
        }
    }
    None
}

fn extrair_id_atributo(xml: &str, tag: &str) -> Option<String> {
    let needle = format!("{tag}");
    let start = xml.find(&needle)?;
    let slice = &xml[start..];
    let id_key = "Id=\"";
    let rel = slice.find(id_key)? + id_key.len();
    let rest = &slice[rel..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

pub fn gerar_chave_acesso_mock() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..44).map(|_| rng.gen_range(0..10).to_string()).collect()
}

pub fn montar_nfse_xml(chave: &str, id_dps: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<NFSe xmlns="http://www.sped.fazenda.gov.br/nfse">
  <infNFSe Id="NFSe{id_dps}">
    <chaveAcesso>{chave}</chaveAcesso>
    <descricao>NFS-e emitida pelo sefin-mock (desenvolvimento local)</descricao>
  </infNFSe>
</NFSe>"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extrai_id_inf_dps() {
        let xml = r#"<DPS><infDPS Id="DPS12345678901234567890123456789012345678901234"></infDPS></DPS>"#;
        assert_eq!(
            extrair_id_dps(xml).as_deref(),
            Some("DPS12345678901234567890123456789012345678901234")
        );
    }
}
