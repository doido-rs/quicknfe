use std::path::{Path, PathBuf};

use base64::Engine;
use uuid::Uuid;

use crate::services::api_error::ApiError;

const MAX_BYTES: usize = 2 * 1024 * 1024;

pub fn gravar_certificado_pfx(
    prestador_id: i64,
    nome_original: &str,
    arquivo_base64: &str,
) -> Result<String, ApiError> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(arquivo_base64.trim())
        .map_err(|_| ApiError::Validation("arquivo_base64 inválido".into()))?;
    if bytes.is_empty() {
        return Err(ApiError::Validation("arquivo do certificado está vazio".into()));
    }
    if bytes.len() > MAX_BYTES {
        return Err(ApiError::Validation(format!(
            "arquivo excede o limite de {} bytes",
            MAX_BYTES
        )));
    }

    let ext = extension_permitida(nome_original)?;
    let dir = certificados_dir(prestador_id)?;
    std::fs::create_dir_all(&dir).map_err(|e| ApiError::Internal(e.to_string()))?;
    let filename = format!("{}.{}", Uuid::new_v4(), ext);
    let path = dir.join(&filename);
    std::fs::write(&path, &bytes).map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(match path.canonicalize() {
        Ok(p) => p.to_string_lossy().into_owned(),
        Err(_) => path.to_string_lossy().into_owned(),
    })
}

fn certificados_dir(prestador_id: i64) -> Result<PathBuf, ApiError> {
    let root = std::env::current_dir().map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(root.join("storage").join("certificados").join(prestador_id.to_string()))
}

/// Remove PFX gravado pelo app (`storage/certificados/…`). Ignora caminhos fora desse diretório.
pub fn tentar_remover_arquivo_certificado(caminho: &str) {
    let normalized = caminho.replace('\\', "/");
    if !normalized.contains("/storage/certificados/") {
        return;
    }
    let path = Path::new(caminho);
    if path.is_file() {
        let _ = std::fs::remove_file(path);
    }
}

fn extension_permitida(nome: &str) -> Result<&'static str, ApiError> {
    let path = Path::new(nome);
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .ok_or_else(|| ApiError::Validation("informe arquivo .pfx ou .p12".into()))?;
    match ext.as_str() {
        "pfx" => Ok("pfx"),
        "p12" => Ok("p12"),
        _ => Err(ApiError::Validation(
            "extensão permitida: .pfx ou .p12".into(),
        )),
    }
}
