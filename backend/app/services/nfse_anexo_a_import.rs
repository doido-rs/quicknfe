//! Importação do Anexo A (municípios IBGE + países ISO2) — planilha oficial NFS-e.

use std::io::Cursor;

use calamine::{Reader, Xlsx};
use doido::model::sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter,
    Set, TransactionTrait,
};

use crate::models::municipio::{ActiveModel as MunicipioActive, Column as MunicipioColumn, Entity as MunicipioEntity};
use crate::models::pais::{ActiveModel as PaisActive, Column as PaisColumn, Entity as PaisEntity};

pub const DEFAULT_XLSX_URL: &str = "https://www.gov.br/nfse/pt-br/biblioteca/documentacao-tecnica/documentacao-atual/anexo_a-municipio_ibge-paises_iso2-v1-00-snnfse-20251210.xlsx";

const SHEET_MUN_IBGE: &str = "TAB.MUN_IBGE";
const SHEET_PAISES_ISO2: &str = "TAB.PAÍS_ISO2";

#[derive(Debug, Clone, Default)]
pub struct ImportStats {
    pub municipios_upserted: u32,
    pub municipios_skipped: u32,
    pub paises_upserted: u32,
    pub paises_skipped: u32,
}

pub fn resolve_download_url(url: Option<&str>) -> &str {
    url.filter(|s| !s.trim().is_empty())
        .unwrap_or(DEFAULT_XLSX_URL)
}

async fn load_xlsx_bytes(url: Option<&str>) -> doido_core::Result<Vec<u8>> {
    let target = resolve_download_url(url);
    if target.starts_with("http://") || target.starts_with("https://") {
        return reqwest::get(target)
            .await
            .map_err(|e| doido_core::anyhow::anyhow!("download anexo A: {e}"))?
            .error_for_status()
            .map_err(|e| doido_core::anyhow::anyhow!("download anexo A HTTP: {e}"))?
            .bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| doido_core::anyhow::anyhow!("read anexo A body: {e}"));
    }
    std::fs::read(target).map_err(|e| doido_core::anyhow::anyhow!("ler anexo A local: {e}"))
}

pub async fn run(db: &DatabaseConnection, url: Option<&str>) -> doido_core::Result<ImportStats> {
    let bytes = load_xlsx_bytes(url).await?;

    let cursor = Cursor::new(bytes.to_vec());
    let mut workbook: Xlsx<_> = Xlsx::new(cursor)
        .map_err(|e| doido_core::anyhow::anyhow!("open xlsx: {e}"))?;

    let txn = db
        .begin()
        .await
        .map_err(|e| doido_core::anyhow::anyhow!("begin txn: {e}"))?;

    let mut stats = ImportStats::default();
    import_municipios(&txn, &mut workbook, &mut stats).await?;
    import_paises(&txn, &mut workbook, &mut stats).await?;

    txn.commit()
        .await
        .map_err(|e| doido_core::anyhow::anyhow!("commit txn: {e}"))?;

    Ok(stats)
}

async fn import_municipios(
    db: &impl ConnectionTrait,
    workbook: &mut Xlsx<Cursor<Vec<u8>>>,
    stats: &mut ImportStats,
) -> doido_core::Result<()> {
    let range = workbook
        .worksheet_range(SHEET_MUN_IBGE)
        .map_err(|e| doido_core::anyhow::anyhow!("sheet {SHEET_MUN_IBGE}: {e}"))?;

    for (idx, row) in range.rows().enumerate() {
        if idx == 0 {
            continue;
        }
        let nome_estado = cell_string(row.first());
        let sigla_raw = cell_string(row.get(1));
        let nome = cell_string(row.get(2));
        let codigo = normalize_ibge(cell_string(row.get(3)));

        if codigo.is_empty() || nome.is_empty() {
            stats.municipios_skipped += 1;
            continue;
        }

        let uf = match resolve_uf(&sigla_raw, &nome_estado) {
            Some(uf) => uf,
            None => {
                stats.municipios_skipped += 1;
                continue;
            }
        };

        upsert_municipio(db, &codigo, &nome, &uf, stats).await?;
    }
    Ok(())
}

async fn import_paises(
    db: &impl ConnectionTrait,
    workbook: &mut Xlsx<Cursor<Vec<u8>>>,
    stats: &mut ImportStats,
) -> doido_core::Result<()> {
    let range = workbook
        .worksheet_range(SHEET_PAISES_ISO2)
        .map_err(|e| doido_core::anyhow::anyhow!("sheet {SHEET_PAISES_ISO2}: {e}"))?;

    for (idx, row) in range.rows().enumerate() {
        if idx == 0 {
            continue;
        }
        let iso2 = cell_string(row.first()).to_uppercase();
        let nome = cell_string(row.get(1));

        if iso2.len() != 2 || nome.is_empty() {
            stats.paises_skipped += 1;
            continue;
        }

        upsert_pais(db, &iso2, &nome, stats).await?;
    }
    Ok(())
}

async fn upsert_municipio(
    db: &impl ConnectionTrait,
    codigo_ibge: &str,
    nome: &str,
    uf: &str,
    stats: &mut ImportStats,
) -> doido_core::Result<()> {
    let existing = MunicipioEntity::find()
        .filter(MunicipioColumn::CodigoIbge.eq(codigo_ibge))
        .one(db)
        .await
        .map_err(|e| doido_core::anyhow::anyhow!("query municipio: {e}"))?;

    if let Some(row) = existing {
        let mut active: MunicipioActive = row.into();
        active.nome = Set(nome.to_string());
        active.uf = Set(uf.to_string());
        active
            .update(db)
            .await
            .map_err(|e| doido_core::anyhow::anyhow!("update municipio: {e}"))?;
    } else {
        MunicipioActive {
            codigo_ibge: Set(codigo_ibge.to_string()),
            nome: Set(nome.to_string()),
            uf: Set(uf.to_string()),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(|e| doido_core::anyhow::anyhow!("insert municipio: {e}"))?;
    }
    stats.municipios_upserted += 1;
    Ok(())
}

async fn upsert_pais(
    db: &impl ConnectionTrait,
    codigo_iso2: &str,
    nome: &str,
    stats: &mut ImportStats,
) -> doido_core::Result<()> {
    let existing = PaisEntity::find()
        .filter(PaisColumn::CodigoIso2.eq(codigo_iso2))
        .one(db)
        .await
        .map_err(|e| doido_core::anyhow::anyhow!("query pais: {e}"))?;

    if let Some(row) = existing {
        let mut active: PaisActive = row.into();
        active.nome = Set(nome.to_string());
        active
            .update(db)
            .await
            .map_err(|e| doido_core::anyhow::anyhow!("update pais: {e}"))?;
    } else {
        PaisActive {
            codigo_iso2: Set(codigo_iso2.to_string()),
            nome: Set(nome.to_string()),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(|e| doido_core::anyhow::anyhow!("insert pais: {e}"))?;
    }
    stats.paises_upserted += 1;
    Ok(())
}

fn cell_string(cell: Option<&calamine::Data>) -> String {
    match cell {
        Some(calamine::Data::String(s)) => s.trim().to_string(),
        Some(calamine::Data::Float(f)) => {
            if f.fract() == 0.0 {
                format!("{:.0}", f)
            } else {
                f.to_string()
            }
        }
        Some(calamine::Data::Int(i)) => i.to_string(),
        Some(calamine::Data::Empty) | None => String::new(),
        Some(other) => other.to_string().trim().to_string(),
    }
}

fn normalize_ibge(raw: String) -> String {
    let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() >= 7 {
        digits[digits.len() - 7..].to_string()
    } else {
        format!("{:0>7}", digits)
    }
}

/// `sigla_uf` = coluna “Sigla UF”; `nome_estado` = coluna “NOME UF”.
pub fn resolve_uf(sigla_uf: &str, nome_estado: &str) -> Option<String> {
    let sigla = sigla_uf.trim();
    if sigla.len() == 2 && sigla.chars().all(|c| c.is_ascii_alphabetic()) {
        return Some(sigla.to_uppercase());
    }
    uf_from_nome_estado(nome_estado).map(str::to_string)
}

fn uf_from_nome_estado(nome: &str) -> Option<&'static str> {
    match normalize_key(nome).as_str() {
        "ACRE" => Some("AC"),
        "ALAGOAS" => Some("AL"),
        "AMAPA" => Some("AP"),
        "AMAZONAS" => Some("AM"),
        "BAHIA" => Some("BA"),
        "CEARA" => Some("CE"),
        "DISTRITO FEDERAL" => Some("DF"),
        "ESPIRITO SANTO" => Some("ES"),
        "GOIAS" => Some("GO"),
        "MARANHAO" => Some("MA"),
        "MATO GROSSO" => Some("MT"),
        "MATO GROSSO DO SUL" => Some("MS"),
        "MINAS GERAIS" => Some("MG"),
        "PARA" => Some("PA"),
        "PARAIBA" => Some("PB"),
        "PARANA" => Some("PR"),
        "PERNAMBUCO" => Some("PE"),
        "PIAUI" => Some("PI"),
        "RIO DE JANEIRO" => Some("RJ"),
        "RIO GRANDE DO NORTE" => Some("RN"),
        "RIO GRANDE DO SUL" => Some("RS"),
        "RONDONIA" => Some("RO"),
        "RORAIMA" => Some("RR"),
        "SANTA CATARINA" => Some("SC"),
        "SAO PAULO" => Some("SP"),
        "SERGIPE" => Some("SE"),
        "TOCANTINS" => Some("TO"),
        _ => None,
    }
}

fn normalize_key(s: &str) -> String {
    let mut t = s.trim().to_uppercase();
    for (from, to) in [
        ('Á', "A"),
        ('À', "A"),
        ('Â', "A"),
        ('Ã', "A"),
        ('É', "E"),
        ('Ê', "E"),
        ('Í', "I"),
        ('Ó', "O"),
        ('Ô', "O"),
        ('Õ', "O"),
        ('Ú', "U"),
        ('Ü', "U"),
        ('Ç', "C"),
    ] {
        t = t.replace(from, to);
    }
    t.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_uf_prefers_sigla() {
        assert_eq!(resolve_uf("sp", "São Paulo").as_deref(), Some("SP"));
    }

    #[test]
    fn resolve_uf_from_estado_when_sigla_missing() {
        assert_eq!(resolve_uf("", "Rondônia").as_deref(), Some("RO"));
        assert_eq!(resolve_uf("  ", "Distrito Federal").as_deref(), Some("DF"));
    }

    #[test]
    fn normalize_ibge_pads() {
        assert_eq!(normalize_ibge("1100015".into()), "1100015");
        assert_eq!(normalize_ibge("3550308".into()), "3550308");
    }

    #[test]
    fn resolve_download_url_ignores_blank_override() {
        assert_eq!(resolve_download_url(None), DEFAULT_XLSX_URL);
        assert_eq!(resolve_download_url(Some("")), DEFAULT_XLSX_URL);
        assert_eq!(resolve_download_url(Some("  ")), DEFAULT_XLSX_URL);
        assert_eq!(
            resolve_download_url(Some("https://example.test/anexo.xlsx")),
            "https://example.test/anexo.xlsx"
        );
    }
}
