use std::collections::HashSet;
use std::path::PathBuf;

use crate::services::nfse::dps_rn::{load_rn_split_rows, load_rn_split_recepcao_rows};
use crate::services::nfse::dps_rn_catalog::ALL_RULES;

#[test]
fn catalog_bijection_425_codigos() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut csv_codes = HashSet::new();
    for row in load_rn_split_rows(&root).expect("nfse csv") {
        if !row.cod_erro.is_empty() {
            csv_codes.insert(row.cod_erro.clone());
        }
    }
    for row in load_rn_split_recepcao_rows(&root).expect("recep csv") {
        if !row.cod_erro.is_empty() {
            csv_codes.insert(row.cod_erro.clone());
        }
    }
    assert_eq!(csv_codes.len(), 425, "CSV deve ter 425 códigos únicos");
    let catalog: HashSet<String> = ALL_RULES.iter().map(|r| r.cod_erro.to_string()).collect();
    assert_eq!(catalog.len(), 425);
    assert_eq!(csv_codes, catalog);
}

#[test]
fn catalog_msgs_oficiais() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut by_cod = std::collections::HashMap::new();
    for row in load_rn_split_rows(&root).expect("csv") {
        by_cod.insert(row.cod_erro.clone(), row.msg_erro.clone());
    }
    for rule in ALL_RULES {
        let msg = by_cod.get(rule.cod_erro).expect("cod in csv");
        assert_eq!(rule.msg_erro, msg.as_str(), "msg {}", rule.cod_erro);
    }
}
