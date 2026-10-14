//! Leitura do `layout_dps.xlsx` (aba `LEIAUTE DPS_NFS-e`) para checklist de cobertura.

use std::fs;
use std::io::Cursor;
use std::path::Path;

use calamine::{Reader, Xlsx};
use serde::{Deserialize, Serialize};

const LEIAUTE_SHEET: &str = "LEIAUTE DPS_NFS-e ";
const INFDPS_PREFIX: &str = "NFSe/infNFSe/DPS/infDPS/";

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
    let path = repo_root.join("layout_dps.xlsx");
    if !path.is_file() {
        return Err(format!("layout_dps.xlsx não encontrado em {}", repo_root.display()));
    }
    let bytes = fs::read(&path).map_err(|e| format!("ler xlsx: {e}"))?;
    let cursor = Cursor::new(bytes);
    let mut workbook: Xlsx<_> = Xlsx::new(cursor).map_err(|e| format!("abrir xlsx: {e}"))?;
    let range = workbook
        .worksheet_range(LEIAUTE_SHEET)
        .map_err(|e| format!("aba {LEIAUTE_SHEET}: {e}"))?;

    let mut rows = Vec::new();
    let mut header: Option<Vec<String>> = None;

    for (i, row) in range.rows().enumerate() {
        let cells: Vec<String> = row.iter().map(cell_to_string).collect();
        if i == 0 {
            header = Some(cells);
            continue;
        }
        let Some(h) = header.as_ref() else { continue };
        let idx_path = h.iter().position(|c| c.contains("CAMINHO"));
        let idx_campo = h.iter().position(|c| c == "CAMPO");
        let idx_ele = h.iter().position(|c| c == "ELE");
        let idx_tipo = h.iter().position(|c| c == "TIPO");
        let idx_ocor = h.iter().position(|c| c == "OCOR.");
        let idx_tam = h.iter().position(|c| c == "TAM.");

        let get = |cells: &[String], idx: Option<usize>| -> String {
            idx.and_then(|i| cells.get(i))
                .map(|s| s.trim().to_string())
                .unwrap_or_default()
        };

        let xml_path = get(&cells, idx_path);
        let campo = get(&cells, idx_campo);
        if xml_path.is_empty() || campo.is_empty() || campo == "-" {
            continue;
        }
        if !xml_path.starts_with(INFDPS_PREFIX) {
            continue;
        }
        rows.push(LayoutFieldRow {
            xml_path,
            campo,
            ele: get(&cells, idx_ele),
            tipo: get(&cells, idx_tipo),
            ocor: get(&cells, idx_ocor),
            tam: get(&cells, idx_tam),
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

fn cell_to_string(cell: &calamine::Data) -> String {
    match cell {
        calamine::Data::String(s) => s.trim().to_string(),
        calamine::Data::Float(f) => {
            if f.fract() == 0.0 {
                format!("{f:.0}")
            } else {
                f.to_string()
            }
        }
        calamine::Data::Int(i) => i.to_string(),
        calamine::Data::Bool(b) => b.to_string(),
        calamine::Data::Empty
        | calamine::Data::Error(_)
        | calamine::Data::DateTime(_)
        | calamine::Data::DateTimeIso(_)
        | calamine::Data::DurationIso(_) => String::new(),
    }
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
        let rows = load_infdps_leiaute(&repo_root()).expect("layout_dps.xlsx");
        assert!(rows.len() > 50, "esperado dezenas de campos infDPS");
        assert!(rows.iter().any(|r| r.campo == "tpAmb"));
        assert!(rows.iter().any(|r| r.campo == "nDPS"));
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
