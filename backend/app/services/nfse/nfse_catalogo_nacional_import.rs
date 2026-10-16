//! Importação opcional do Anexo B (lista nacional). Falha silenciosa se URL indisponível.

use std::io::Cursor;

use calamine::{Reader, Xlsx};
use doido::model::sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter,
    Set,
};

use crate::models::cna::{ActiveModel as CnaActive, Column as CnaColumn, Entity as CnaEntity};
use crate::services::nfse::codigo_servico::normalize_c_trib_nac;

#[derive(Debug, Clone, Default)]
pub struct CatalogoNacionalImportStats {
    pub upserted: u32,
    pub skipped: u32,
}

/// Tenta baixar Anexo B; se URL vazia ou download falhar, retorna stats zeradas.
pub async fn run(
    db: &DatabaseConnection,
    url: Option<&str>,
) -> doido_core::Result<CatalogoNacionalImportStats> {
    let Some(u) = url.filter(|s| !s.trim().is_empty()) else {
        return Ok(CatalogoNacionalImportStats::default());
    };
    let bytes = match download(u).await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Anexo B indisponível ({u}): {e}");
            return Ok(CatalogoNacionalImportStats::default());
        }
    };
    let mut stats = CatalogoNacionalImportStats::default();
    import_from_bytes(db, &bytes, &mut stats).await?;
    Ok(stats)
}

async fn download(url: &str) -> doido_core::Result<Vec<u8>> {
    reqwest::get(url)
        .await
        .map_err(|e| doido_core::anyhow::anyhow!("download anexo B: {e}"))?
        .error_for_status()
        .map_err(|e| doido_core::anyhow::anyhow!("download anexo B HTTP: {e}"))?
        .bytes()
        .await
        .map(|b| b.to_vec())
        .map_err(|e| doido_core::anyhow::anyhow!("read anexo B body: {e}"))
}

async fn import_from_bytes(
    db: &DatabaseConnection,
    bytes: &[u8],
    stats: &mut CatalogoNacionalImportStats,
) -> doido_core::Result<()> {
    let cursor = Cursor::new(bytes.to_vec());
    let mut workbook: Xlsx<_> =
        Xlsx::new(cursor).map_err(|e| doido_core::anyhow::anyhow!("open anexo B: {e}"))?;
    let sheet_name = workbook
        .sheet_names()
        .iter()
        .find(|n| n.to_uppercase().contains("LISTA") || n.to_uppercase().contains("SERV"))
        .cloned()
        .or_else(|| workbook.sheet_names().first().cloned())
        .ok_or_else(|| doido_core::anyhow::anyhow!("anexo B sem abas"))?;

    let range = workbook
        .worksheet_range(&sheet_name)
        .map_err(|e| doido_core::anyhow::anyhow!("sheet {sheet_name}: {e}"))?;

    for (idx, row) in range.rows().enumerate() {
        if idx == 0 {
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
        merge_descricao(db, &codigo, &descricao).await?;
        stats.upserted += 1;
    }
    Ok(())
}

async fn merge_descricao(
    db: &impl ConnectionTrait,
    codigo: &str,
    descricao: &str,
) -> doido_core::Result<()> {
    let existing = CnaEntity::find()
        .filter(CnaColumn::Codigo.eq(codigo))
        .one(db)
        .await
        .map_err(|e| doido_core::anyhow::anyhow!("query cna: {e}"))?;

    if let Some(row) = existing {
        let mut active: CnaActive = row.into();
        active.descricao = Set(descricao.to_string());
        active
            .update(db)
            .await
            .map_err(|e| doido_core::anyhow::anyhow!("update cna: {e}"))?;
    } else {
        CnaActive {
            codigo: Set(codigo.to_string()),
            descricao: Set(descricao.to_string()),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(|e| doido_core::anyhow::anyhow!("insert cna: {e}"))?;
    }
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
