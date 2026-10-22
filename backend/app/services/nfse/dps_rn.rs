//! Regras de negócio importadas de `layout_dps.csv` (RN DPS + RN recepção).

use std::path::Path;

use super::dps_layout_csv::{
    column_index, column_index_contains, find_layout_header, get_field, read_csv_records,
};

const RN_CSV: &str = "layout_dps.csv";
const RN_SPLIT_NFSE: &str = "rn_layout_dps_nfse.csv";
const RN_SPLIT_RECEP: &str = "rn_layout_recepcao_dps.csv";
const SHEET_RN_DPS: &str = "RN DPS_NFS-e";
const SHEET_RN_RECEPCAO: &str = "RN_RECEPCAO_DPS";

#[derive(Debug, Clone)]
pub struct RnRow {
    pub xml_path: String,
    pub campo: String,
    pub descricao: String,
    pub aplic: String,
    pub efeito: String,
    pub cod_erro: String,
    pub msg_erro: String,
    pub recv_dps: String,
}

fn load_rn_split_file(repo_root: &Path, filename: &str) -> Result<Vec<RnRow>, String> {
    let path = repo_root.join(filename);
    let records = read_csv_records(&path)?;
    let mut rows = Vec::new();
    for rec in records {
        if rec.len() < 8 {
            continue;
        }
        let aplic = rec[5].trim();
        let efeito = rec[6].trim();
        if !aplic.to_lowercase().contains("obrig") || !efeito.to_lowercase().starts_with("rej") {
            continue;
        }
        let cod = rec[7].trim();
        if cod.is_empty() {
            continue;
        }
        rows.push(RnRow {
            xml_path: rec.get(1).map(|s| s.trim().to_string()).unwrap_or_default(),
            campo: rec.get(2).map(|s| s.trim().to_string()).unwrap_or_default(),
            descricao: rec.get(3).map(|s| s.trim().to_string()).unwrap_or_default(),
            aplic: aplic.to_string(),
            efeito: efeito.to_string(),
            cod_erro: cod.to_string(),
            msg_erro: rec.get(8).map(|s| s.trim().to_string()).unwrap_or_default(),
            recv_dps: rec.get(10).map(|s| s.trim().to_uppercase()).unwrap_or_default(),
        });
    }
    Ok(rows)
}

pub fn load_rn_split_rows(repo_root: &Path) -> Result<Vec<RnRow>, String> {
    load_rn_split_file(repo_root, RN_SPLIT_NFSE)
}

pub fn load_rn_split_recepcao_rows(repo_root: &Path) -> Result<Vec<RnRow>, String> {
    load_rn_split_file(repo_root, RN_SPLIT_RECEP)
}

fn load_rn_rows_filtered(repo_root: &Path, origem: Option<&str>) -> Result<Vec<RnRow>, String> {
    let path = repo_root.join(RN_CSV);
    let records = read_csv_records(&path)?;
    let (header_idx, headers) = find_layout_header(&records)
        .ok_or_else(|| format!("cabeçalho RN ausente em {}", path.display()))?;

    let idx_origem = column_index(&headers, "ORIGEM");
    let idx_path = column_index_contains(&headers, "CAMINHO");
    let idx_campo = column_index(&headers, "CAMPO");
    // Colunas após re-export: descrição costuma ser a 4ª coluna de negócio; usamos nomes quando existirem.
    let idx_aplic = column_index(&headers, "APLIC.");
    let idx_efeito = column_index(&headers, "EFEITO");
    let idx_cod = column_index_contains(&headers, "CÓD");
    let idx_msg = column_index_contains(&headers, "MSG");

    let desc_fallback = 4usize;

    let mut rows = Vec::new();
    for rec in records.iter().skip(header_idx + 1) {
        if let Some(o) = origem {
            if idx_origem.is_some() {
                let sheet = get_field(rec, idx_origem);
                if sheet != o {
                    continue;
                }
            }
        }

        let xml_path = get_field(rec, idx_path);
        let campo = get_field(rec, idx_campo);
        if xml_path.is_empty() && campo.is_empty() {
            continue;
        }
        let descricao = idx_campo
            .and_then(|c| rec.get(c + 1))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| get_field(rec, Some(desc_fallback)));

        rows.push(RnRow {
            xml_path,
            campo,
            descricao,
            aplic: get_field(rec, idx_aplic),
            efeito: get_field(rec, idx_efeito),
            cod_erro: get_field(rec, idx_cod),
            msg_erro: get_field(rec, idx_msg),
            recv_dps: String::new(),
        });
    }
    Ok(rows)
}

pub fn load_rn_dps_rows(repo_root: &Path) -> Result<Vec<RnRow>, String> {
    load_rn_rows_filtered(repo_root, Some(SHEET_RN_DPS))
}

pub fn load_rn_recepcao_rows(repo_root: &Path) -> Result<Vec<RnRow>, String> {
    load_rn_rows_filtered(repo_root, Some(SHEET_RN_RECEPCAO))
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

    #[test]
    fn rn_split_csv_carrega() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        let rows = load_rn_split_rows(&root).expect("split nfse");
        assert!(rows.len() >= 400);
    }
}
