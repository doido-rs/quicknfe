//! Leitura do `layout_dps_leaiute.csv` para checklist de cobertura `infDPS`.

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::dps_layout_csv::{
    column_index, column_index_contains, find_layout_header, get_field, read_csv_records,
};

const INFDPS_PREFIX: &str = "NFSe/infNFSe/DPS/infDPS/";
const LEIAUTE_CSV: &str = "layout_dps_leaiute.csv";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LayoutFieldRow {
    pub xml_path: String,
    pub campo: String,
    pub ele: String,
    pub tipo: String,
    pub ocor: String,
    pub tam: String,
}

/// Campos 1-1 ou 1-n sob `infDPS` (exclui grupos raiz NFSe processada).
pub fn load_infdps_leiaute(repo_root: &Path) -> Result<Vec<LayoutFieldRow>, String> {
    let path = repo_root.join(LEIAUTE_CSV);
    let records = read_csv_records(&path)?;
    let (header_idx, headers) = find_layout_header(&records)
        .ok_or_else(|| format!("cabeçalho leiaute ausente em {}", path.display()))?;

    let idx_path = column_index_contains(&headers, "CAMINHO")
        .ok_or_else(|| "coluna CAMINHO NO XML".to_string())?;
    let idx_campo = column_index(&headers, "CAMPO").ok_or_else(|| "coluna CAMPO".to_string())?;
    let idx_ele = column_index(&headers, "ELE").ok_or_else(|| "coluna ELE".to_string())?;
    let idx_tipo = column_index(&headers, "TIPO").ok_or_else(|| "coluna TIPO".to_string())?;
    let idx_ocor = column_index(&headers, "OCOR.").ok_or_else(|| "coluna OCOR.".to_string())?;
    let idx_tam = column_index(&headers, "TAM.").ok_or_else(|| "coluna TAM.".to_string())?;

    let mut rows = Vec::new();
    for rec in records.iter().skip(header_idx + 1) {
        let xml_path = get_field(rec, Some(idx_path));
        let campo = get_field(rec, Some(idx_campo));
        if xml_path.is_empty() || campo.is_empty() || campo == "-" {
            continue;
        }
        if !xml_path.starts_with(INFDPS_PREFIX) {
            continue;
        }
        rows.push(LayoutFieldRow {
            xml_path,
            campo,
            ele: get_field(rec, Some(idx_ele)),
            tipo: get_field(rec, Some(idx_tipo)),
            ocor: get_field(rec, Some(idx_ocor)),
            tam: get_field(rec, Some(idx_tam)),
        });
    }
    Ok(rows)
}

pub fn mandatory_infdps_fields(rows: &[LayoutFieldRow]) -> Vec<&LayoutFieldRow> {
    rows.iter()
        .filter(|r| {
            r.ocor.starts_with("1-1") || r.ocor.starts_with("1-n") || r.ocor == "1..n"
        })
        .filter(|r| r.ele == "E" || r.ele == "A" || r.ele == "ID")
        .collect()
}

/// Cenários ainda sem modelo de dados/UI (Tier B — ver `docs/nfse/dps-presenca-mapped.md`).
const TIER_B_PATH_MARKERS: &[&str] = &[
    "infDPS/subst",
    "serv/comExt",
    "serv/atvEvento",
    "vDedRed",
    "exigSusp",
    "IBSCBS",
];

pub fn is_tier_a_owned(row: &LayoutFieldRow) -> bool {
    !TIER_B_PATH_MARKERS
        .iter()
        .any(|marker| row.xml_path.contains(marker))
}

/// Campos obrigatórios do leiaute que o QuickNFe cobre no fluxo SEFIN-first atual (Tier A).
pub fn mandatory_infdps_fields_owned(rows: &[LayoutFieldRow]) -> Vec<&LayoutFieldRow> {
    mandatory_infdps_fields(rows)
        .into_iter()
        .filter(|r| is_tier_a_owned(r))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
    }

    #[test]
    fn layout_dps_carrega_infdps() {
        let rows = load_infdps_leiaute(&repo_root()).expect("layout_dps_leaiute.csv");
        assert!(rows.len() > 50, "esperado dezenas de campos infDPS");
        assert!(rows.iter().any(|r| r.campo == "tpAmb"));
        assert!(rows.iter().any(|r| r.campo == "nDPS"));
    }

    #[test]
    fn tier_a_exclui_subst_e_ibscbs() {
        let rows = load_infdps_leiaute(&repo_root()).unwrap();
        let owned = mandatory_infdps_fields_owned(&rows);
        assert!(
            !owned.iter().any(|r| r.campo == "chSubstda"),
            "subst deve ser Tier B"
        );
        assert!(
            !owned.iter().any(|r| r.xml_path.contains("IBSCBS")),
            "IBSCBS deve ser Tier B"
        );
    }

    #[test]
    fn layout_tem_campos_obrigatorios_identificacao() {
        let rows = load_infdps_leiaute(&repo_root()).unwrap();
        for tag in ["tpAmb", "dhEmi", "dCompet", "serie", "nDPS", "cLocEmi"] {
            assert!(
                rows.iter().any(|r| r.campo == tag),
                "campo obrigatório {tag} ausente no leiaute"
            );
        }
    }
}
