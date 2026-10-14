//! Regras de negócio importadas da aba `RN DPS_NFS-e` do layout oficial.

use std::fs;
use std::io::Cursor;
use std::path::Path;

use calamine::{Reader, Xlsx};

const RN_SHEET: &str = "RN DPS_NFS-e";
const RN_RECEPCAO_SHEET: &str = "RN_RECEPCAO_DPS";

#[derive(Debug, Clone)]
pub struct RnRow {
    pub xml_path: String,
    pub campo: String,
    pub descricao: String,
}

pub fn load_rn_dps_rows(repo_root: &Path) -> Result<Vec<RnRow>, String> {
    let path = repo_root.join("layout_dps.xlsx");
    let bytes = fs::read(&path).map_err(|e| format!("ler layout: {e}"))?;
    let cursor = Cursor::new(bytes);
    let mut workbook: Xlsx<_> = Xlsx::new(cursor).map_err(|e| format!("xlsx: {e}"))?;
    let range = workbook
        .worksheet_range(RN_SHEET)
        .map_err(|e| format!("aba {RN_SHEET}: {e}"))?;

    let mut rows = Vec::new();
    for (i, row) in range.rows().enumerate() {
        if i == 0 {
            continue;
        }
        let cells: Vec<String> = row.iter().map(cell_to_string).collect();
        if cells.is_empty() {
            continue;
        }
        let xml_path = cells.first().cloned().unwrap_or_default();
        let campo = cells.get(1).cloned().unwrap_or_default();
        if xml_path.is_empty() && campo.is_empty() {
            continue;
        }
        let descricao = cells.get(3).cloned().unwrap_or_default();
        rows.push(RnRow {
            xml_path,
            campo,
            descricao,
        });
    }
    Ok(rows)
}

pub fn load_rn_recepcao_rows(repo_root: &Path) -> Result<Vec<RnRow>, String> {
    let path = repo_root.join("layout_dps.xlsx");
    let bytes = fs::read(&path).map_err(|e| format!("ler layout: {e}"))?;
    let cursor = Cursor::new(bytes);
    let mut workbook: Xlsx<_> = Xlsx::new(cursor).map_err(|e| format!("xlsx: {e}"))?;
    let range = workbook
        .worksheet_range(RN_RECEPCAO_SHEET)
        .map_err(|e| format!("aba {RN_RECEPCAO_SHEET}: {e}"))?;

    let mut rows = Vec::new();
    for (i, row) in range.rows().enumerate() {
        if i == 0 {
            continue;
        }
        let cells: Vec<String> = row.iter().map(cell_to_string).collect();
        if cells.is_empty() {
            continue;
        }
        let xml_path = cells.first().cloned().unwrap_or_default();
        let campo = cells.get(1).cloned().unwrap_or_default();
        if xml_path.is_empty() && campo.is_empty() {
            continue;
        }
        let descricao = cells.get(3).cloned().unwrap_or_default();
        rows.push(RnRow {
            xml_path,
            campo,
            descricao,
        });
    }
    Ok(rows)
}

fn cell_to_string(cell: &calamine::Data) -> String {
    match cell {
        calamine::Data::String(s) => s.trim().to_string(),
        calamine::Data::Float(f) => f.to_string(),
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

    #[test]
    fn rn_sheet_carrega() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        let rows = load_rn_dps_rows(&root).expect("rn rows");
        assert!(!rows.is_empty());
    }

    #[test]
    fn rn_recepcao_sheet_carrega() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        let rows = load_rn_recepcao_rows(&root).expect("rn recepcao");
        assert!(!rows.is_empty());
    }
}
