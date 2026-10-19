//! Download HTTP(S) de planilhas Anexo A (municípios / países).

const DEFAULT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(120);

pub async fn download_bytes(url: &str) -> doido_core::Result<Vec<u8>> {
    let target = url.trim();
    if !target.starts_with("http://") && !target.starts_with("https://") {
        return std::fs::read(target).map_err(|e| doido_core::anyhow::anyhow!("ler anexo A local: {e}"));
    }

    let client = reqwest::Client::builder()
        .timeout(DEFAULT_TIMEOUT)
        .build()
        .map_err(|e| doido_core::anyhow::anyhow!("cliente HTTP anexo A: {e}"))?;

    let response = client
        .get(target)
        .send()
        .await
        .map_err(|e| doido_core::anyhow::anyhow!("download anexo A: {e}"))?
        .error_for_status()
        .map_err(|e| doido_core::anyhow::anyhow!("download anexo A HTTP: {e}"))?;

    response
        .bytes()
        .await
        .map(|b| b.to_vec())
        .map_err(|e| doido_core::anyhow::anyhow!("read anexo A body: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn download_local_file() {
        let path = std::env::temp_dir().join(format!(
            "quicknfe_anexo_a_test_{}.bin",
            std::process::id()
        ));
        std::fs::write(&path, b"PK-test").unwrap();
        let bytes = download_bytes(path.to_str().unwrap()).await.unwrap();
        assert_eq!(bytes, b"PK-test");
        let _ = std::fs::remove_file(path);
    }
}
