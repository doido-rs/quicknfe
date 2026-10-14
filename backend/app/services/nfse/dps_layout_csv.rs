//! Leitura de `layout_dps_leaiute.csv` e `layout_dps.csv` (UTF-8, campos multilinha).

use std::fs;
use std::path::Path;

use csv::StringRecord;

pub fn read_csv_records(path: &Path) -> Result<Vec<StringRecord>, String> {
    if !path.is_file() {
        return Err(format!("arquivo não encontrado: {}", path.display()));
    }
    let data = fs::read(path).map_err(|e| format!("ler {}: {e}", path.display()))?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(data.as_slice());
    let mut out = Vec::new();
    for result in rdr.records() {
        out.push(result.map_err(|e| format!("csv {}: {e}", path.display()))?);
    }
    Ok(out)
}

/// Localiza a linha de cabeçalho que contém `CAMINHO NO XML` e `CAMPO`.
pub fn find_layout_header(records: &[StringRecord]) -> Option<(usize, Vec<String>)> {
    for (i, rec) in records.iter().enumerate() {
        let headers: Vec<String> = rec.iter().map(|s| s.trim().to_string()).collect();
        let has_path = headers.iter().any(|h| h.contains("CAMINHO"));
        let has_campo = headers.iter().any(|h| h == "CAMPO");
        if has_path && has_campo {
            return Some((i, headers));
        }
    }
    None
}

pub fn column_index(headers: &[String], name: &str) -> Option<usize> {
    headers.iter().position(|h| h == name)
}

pub fn column_index_contains(headers: &[String], substr: &str) -> Option<usize> {
    headers.iter().position(|h| h.contains(substr))
}

pub fn get_field(rec: &StringRecord, idx: Option<usize>) -> String {
    idx.and_then(|i| rec.get(i))
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}
