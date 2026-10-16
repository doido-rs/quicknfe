//! Importação da lista nacional + regras de incidência (aba MUN.INCID_INFO.SERV.).

use std::io::Cursor;
use std::path::Path;

use calamine::{Reader, Xlsx};
use chrono::Utc;
use doido::model::sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter,
    Set,
};

use crate::models::cna::{ActiveModel as CnaActive, Column as CnaColumn, Entity as CnaEntity};
use crate::services::nfse::codigo_servico::normalize_c_trib_nac;

pub const DEFAULT_INCIDENCIA_XLSX: &str = "layout_dps.xlsx";
const SHEET: &str = "MUN.INCID_INFO.SERV.";

#[derive(Debug, Clone, Default)]
pub struct IncidenciaImportStats {
    pub upserted: u32,
    pub skipped: u32,
}

pub async fn run(
    db: &DatabaseConnection,
    url: Option<&str>,
) -> doido_core::Result<IncidenciaImportStats> {
    let bytes = load_xlsx_bytes(url).await?;
    let mut stats = IncidenciaImportStats::default();
    import_from_bytes(db, &bytes, &mut stats).await?;
    Ok(stats)
}

async fn load_xlsx_bytes(url: Option<&str>) -> doido_core::Result<Vec<u8>> {
    if let Some(u) = url.filter(|s| !s.trim().is_empty()) {
        if u.starts_with("http://") || u.starts_with("https://") {
            return reqwest::get(u)
                .await
                .map_err(|e| doido_core::anyhow::anyhow!("download incidência xlsx: {e}"))?
                .error_for_status()
                .map_err(|e| doido_core::anyhow::anyhow!("download incidência HTTP: {e}"))?
                .bytes()
                .await
                .map(|b| b.to_vec())
                .map_err(|e| doido_core::anyhow::anyhow!("read incidência body: {e}"));
        }
        return std::fs::read(u)
            .map_err(|e| doido_core::anyhow::anyhow!("ler incidência xlsx local: {e}"));
    }
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(DEFAULT_INCIDENCIA_XLSX);
    std::fs::read(&path).map_err(|e| {
        doido_core::anyhow::anyhow!("ler {path:?}: {e} — defina NFSE_INCIDENCIA_XLSX_URL")
    })
}

async fn import_from_bytes(
    db: &DatabaseConnection,
    bytes: &[u8],
    stats: &mut IncidenciaImportStats,
) -> doido_core::Result<()> {
    let cursor = Cursor::new(bytes.to_vec());
    let mut workbook: Xlsx<_> = Xlsx::new(cursor)
        .map_err(|e| doido_core::anyhow::anyhow!("open incidência xlsx: {e}"))?;
    let range = workbook
        .worksheet_range(SHEET)
        .map_err(|e| doido_core::anyhow::anyhow!("sheet {SHEET}: {e}"))?;

    let now = Utc::now().naive_utc();
    for (idx, row) in range.rows().enumerate() {
        if idx < 4 {
            continue;
        }
        let code_raw = cell_string(row.first());
        let Some(codigo) = normalize_c_trib_nac(&code_raw) else {
            stats.skipped += 1;
            continue;
        };
        let descricao = cell_string(row.get(1));
        if descricao.is_empty() {
            stats.skipped += 1;
            continue;
        }
        let incid_ep = flag_x(row.get(2));
        let incid_lp = flag_x(row.get(3));
        let incid_et = flag_x(row.get(4));
        let grupo_obra = flag_x(row.get(6));
        let grupo_atv = flag_x(row.get(7));

        upsert_cna(
            db,
            &codigo,
            &descricao,
            incid_ep,
            incid_lp,
            incid_et,
            grupo_obra,
            grupo_atv,
            now,
        )
        .await?;
        stats.upserted += 1;
    }
    Ok(())
}

async fn upsert_cna(
    db: &impl ConnectionTrait,
    codigo: &str,
    descricao: &str,
    incid_ep: i8,
    incid_lp: i8,
    incid_et: i8,
    grupo_obra: i8,
    grupo_atv_evento: i8,
    synced_at: chrono::NaiveDateTime,
) -> doido_core::Result<()> {
    let existing = CnaEntity::find()
        .filter(CnaColumn::Codigo.eq(codigo))
        .one(db)
        .await
        .map_err(|e| doido_core::anyhow::anyhow!("query cna: {e}"))?;

    if let Some(row) = existing {
        let mut active: CnaActive = row.into();
        active.descricao = Set(descricao.to_string());
        active.incid_ep = Set(incid_ep);
        active.incid_lp = Set(incid_lp);
        active.incid_et = Set(incid_et);
        active.grupo_obra = Set(grupo_obra);
        active.grupo_atv_evento = Set(grupo_atv_evento);
        active.synced_at = Set(Some(synced_at));
        active
            .update(db)
            .await
            .map_err(|e| doido_core::anyhow::anyhow!("update cna: {e}"))?;
    } else {
        CnaActive {
            codigo: Set(codigo.to_string()),
            descricao: Set(descricao.to_string()),
            incid_ep: Set(incid_ep),
            incid_lp: Set(incid_lp),
            incid_et: Set(incid_et),
            grupo_obra: Set(grupo_obra),
            grupo_atv_evento: Set(grupo_atv_evento),
            synced_at: Set(Some(synced_at)),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(|e| doido_core::anyhow::anyhow!("insert cna: {e}"))?;
    }
    Ok(())
}

fn flag_x(cell: Option<&calamine::Data>) -> i8 {
    if cell_string(cell).eq_ignore_ascii_case("X") {
        1
    } else {
        0
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flag_x_detects_mark() {
        assert_eq!(flag_x(Some(&calamine::Data::String("X".into()))), 1);
        assert_eq!(flag_x(Some(&calamine::Data::String("-".into()))), 0);
    }
}
