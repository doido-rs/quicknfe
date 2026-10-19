//! Cliente HTTP ADN — parametrização municipal (`/parametrizacao/...`).
//!
//! OpenAPI: `{base}/parametrizacao/docs` — sync municipal usa
//! `GET /{codigoMunicipio}/{codigoServico}/historicoaliquotas`.

mod codigo;
mod pfx;

use doido::model::sea_orm::prelude::Decimal;
use std::collections::HashSet;
use std::str::FromStr;
use std::time::Duration;

use reqwest::Client;

use crate::connectors::mtls::build_mtls_client_from_pfx_bytes;

use pfx::{adn_pfx_password, load_adn_pfx_bytes};
use crate::services::api_error::ApiError;
use crate::settings;

pub use codigo::{
    format_codigo_servico_adn, normalize_c_trib_mun, parse_codigo_parametrizacao,
};

#[derive(Debug, Clone)]
pub struct AliquotaParametrizacao {
    pub codigo_parametrizacao: String,
    pub codigo_trib_nac: String,
    pub codigo_trib_mun: String,
    pub aliquota: Option<Decimal>,
    pub vigencia_inicio: Option<chrono::NaiveDate>,
    pub vigencia_fim: Option<chrono::NaiveDate>,
}

/// Mock/local ADN não exige mTLS (mesma regra do CNC).
pub fn adn_usa_mtls(base_url: &str) -> bool {
    let lower = base_url.to_ascii_lowercase();
    !(lower.contains("127.0.0.1") || lower.contains("localhost"))
}

/// URL do histórico de alíquotas (OpenAPI `Aliquotas` → `historicoaliquotas`).
pub fn historico_aliquotas_url(codigo_ibge: &str, codigo_servico: &str) -> String {
    let base = settings::get().adn.base_url_normalized();
    format!(
        "{base}/parametrizacao/{}/{}/historicoaliquotas",
        codigo_ibge.trim(),
        codigo_servico.trim()
    )
}

/// URL alíquota na competência (`/{competencia}/aliquota`, `competencia` ISO date-time).
pub fn aliquota_competencia_url(
    codigo_ibge: &str,
    codigo_servico: &str,
    competencia_iso: &str,
) -> String {
    let base = settings::get().adn.base_url_normalized();
    let comp = urlencoding::encode(competencia_iso.trim());
    format!(
        "{base}/parametrizacao/{}/{}/{comp}/aliquota",
        codigo_ibge.trim(),
        codigo_servico.trim()
    )
}

/// Consulta parametrização por município, iterando códigos nacionais do catálogo (`cnas`).
pub async fn get_aliquotas_municipio(
    codigo_ibge: &str,
    codigos_nacionais: &[String],
) -> Result<Vec<AliquotaParametrizacao>, ApiError> {
    let ibge = codigo_ibge.trim();
    let sleep_ms = settings::get().nfse_catalog.sleep_ms;
    let client = http_client()?;
    let mut out = Vec::new();
    let mut seen = HashSet::new();

    for raw in codigos_nacionais {
        let rows = fetch_historico_for_nacional(&client, ibge, raw, sleep_ms).await?;
        for row in rows {
            let key = (row.codigo_trib_nac.clone(), row.codigo_trib_mun.clone());
            if seen.insert(key) {
                out.push(row);
            }
        }
    }
    Ok(out)
}

/// Histórico ADN por CNA nacional. Municípios como RJ publicam em `…001`, não `…000`.
async fn fetch_historico_for_nacional(
    client: &Client,
    ibge: &str,
    raw_nacional: &str,
    sleep_ms: u64,
) -> Result<Vec<AliquotaParametrizacao>, ApiError> {
    let desdobro_max = settings::get().nfse_catalog.desdobro_scan_max;
    let desdobro_stop = settings::get().nfse_catalog.desdobro_scan_stop_after_misses;
    let mut rows = Vec::new();

    if let Some(codigo) = format_codigo_servico_adn(raw_nacional, "000") {
        rows = fetch_historico_aliquotas(client, ibge, &codigo).await?;
        if sleep_ms > 0 {
            tokio::time::sleep(Duration::from_millis(sleep_ms)).await;
        }
    }

    if rows.is_empty() {
        let max = desdobro_max.max(1);
        let stop_after = desdobro_stop.max(1);
        let mut misses = 0u32;
        for n in 1..=max {
            let mun = format!("{n:03}");
            let Some(codigo) = format_codigo_servico_adn(raw_nacional, &mun) else {
                continue;
            };
            let batch = fetch_historico_aliquotas(client, ibge, &codigo).await?;
            if batch.is_empty() {
                misses += 1;
                if misses >= stop_after {
                    break;
                }
            } else {
                misses = 0;
                rows.extend(batch);
            }
            if sleep_ms > 0 {
                tokio::time::sleep(Duration::from_millis(sleep_ms)).await;
            }
        }
    }

    Ok(rows)
}

async fn fetch_historico_aliquotas(
    client: &Client,
    ibge: &str,
    codigo_servico: &str,
) -> Result<Vec<AliquotaParametrizacao>, ApiError> {
    let url = historico_aliquotas_url(ibge, codigo_servico);

    let response = client
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| ApiError::Internal(format!("falha HTTP ADN ({url}): {e}")))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| ApiError::Internal(format!("corpo ADN: {e}")))?;

    if status.as_u16() == 404 {
        return Ok(Vec::new());
    }
    if !status.is_success() {
        return Err(ApiError::Internal(format!(
            "ADN parametrizacao HTTP {}: {}",
            status.as_u16(),
            truncate(&body, 500)
        )));
    }

    parse_historico_aliquotas_body(&body)
}

fn http_client() -> Result<Client, ApiError> {
    let adn = settings::get().adn.base_url_normalized();
    if adn_usa_mtls(&adn) {
        if let Some(bytes) = load_adn_pfx_bytes()? {
            let senha = adn_pfx_password();
            return build_mtls_client_from_pfx_bytes(&bytes, &senha);
        }
    }
    Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| ApiError::Internal(format!("cliente HTTP ADN: {e}")))
}

/// Parse resposta OpenAPI `ResultadoConsultaAliquotas` (histórico / alíquota).
pub fn parse_historico_aliquotas_body(body: &str) -> Result<Vec<AliquotaParametrizacao>, ApiError> {
    parse_aliquotas_json(body)
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max])
    }
}

fn parse_aliquotas_json(body: &str) -> Result<Vec<AliquotaParametrizacao>, ApiError> {
    if body.trim().is_empty() {
        return Ok(Vec::new());
    }
    let value: serde_json::Value = serde_json::from_str(body)
        .map_err(|e| ApiError::Internal(format!("JSON ADN inválido: {e}")))?;

    if let Some(rows) = parse_aliquotas_map(&value) {
        if !rows.is_empty() {
            return Ok(rows);
        }
    }

    let items = extract_items(&value);
    let mut out = Vec::new();
    for item in items {
        if let Some(row) = map_item(&item, None) {
            out.push(row);
        }
    }
    Ok(out)
}

/// Formato oficial: `{ "aliquotas": { "01.01.01.001": [ { "Aliq": 2, "DtIni": "..." } ] } }`.
fn parse_aliquotas_map(value: &serde_json::Value) -> Option<Vec<AliquotaParametrizacao>> {
    let map = value.get("aliquotas")?.as_object()?;
    let mut out = Vec::new();
    for (codigo_servico, entries) in map {
        let arr = entries.as_array().cloned().unwrap_or_default();
        for item in arr {
            if let Some(row) = map_item(&item, Some(codigo_servico.as_str())) {
                out.push(row);
            }
        }
    }
    Some(out)
}

fn extract_items(value: &serde_json::Value) -> Vec<serde_json::Value> {
    if value.is_array() {
        return value.as_array().cloned().unwrap_or_default();
    }
    const KEYS: &[&str] = &[
        "aliquotas",
        "listaAliquotas",
        "parametros",
        "itens",
        "servicos",
    ];
    for key in KEYS {
        if let Some(arr) = value.get(*key).and_then(|v| v.as_array()) {
            return arr.clone();
        }
    }
    if let Some(obj) = value.as_object() {
        for v in obj.values() {
            if v.is_array() {
                return v.as_array().cloned().unwrap_or_default();
            }
        }
    }
    Vec::new()
}

fn map_item(item: &serde_json::Value, codigo_override: Option<&str>) -> Option<AliquotaParametrizacao> {
    let codigo_raw = codigo_override
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .or_else(|| {
            item.get("codigoServico")
                .or_else(|| item.get("codigo"))
                .or_else(|| item.get("codServ"))
                .or_else(|| item.get("codigoParametrizacao"))
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
        })?;

    let (codigo_trib_nac, codigo_trib_mun) = parse_codigo_parametrizacao(&codigo_raw)?;

    let aliquota = field_decimal(item, &["aliquota", "pAliq", "Aliq", "aliq"]);
    let vigencia_inicio = parse_date_field(item, &["dtIni", "DtIni", "dataInicio", "vigenciaInicio"]);
    let vigencia_fim = parse_date_field(item, &["dtFim", "DtFim", "dataFim", "vigenciaFim"]);

    Some(AliquotaParametrizacao {
        codigo_parametrizacao: codigo_raw,
        codigo_trib_nac,
        codigo_trib_mun,
        aliquota,
        vigencia_inicio,
        vigencia_fim,
    })
}

fn field_decimal(item: &serde_json::Value, keys: &[&str]) -> Option<Decimal> {
    for key in keys {
        if let Some(v) = item.get(*key) {
            if let Some(d) = parse_decimal(v) {
                return Some(d);
            }
        }
    }
    None
}

fn parse_decimal(v: &serde_json::Value) -> Option<Decimal> {
    match v {
        serde_json::Value::Number(n) => Decimal::from_str(&n.to_string()).ok(),
        serde_json::Value::String(s) => Decimal::from_str(s.trim()).ok(),
        _ => None,
    }
}

fn parse_date_field(item: &serde_json::Value, keys: &[&str]) -> Option<chrono::NaiveDate> {
    for key in keys {
        if let Some(s) = item.get(*key).and_then(|v| v.as_str()) {
            if let Ok(d) = chrono::NaiveDate::parse_from_str(&s[..s.len().min(10)], "%Y-%m-%d") {
                return Some(d);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adn_mock_nao_exige_mtls() {
        assert!(!adn_usa_mtls("http://127.0.0.1:8090"));
        assert!(adn_usa_mtls("https://adn.producaorestrita.nfse.gov.br"));
    }

    #[test]
    fn historico_url_matches_openapi() {
        settings::init();
        let url = historico_aliquotas_url("3304557", "01.01.01.000");
        assert!(url.ends_with("/parametrizacao/3304557/01.01.01.000/historicoaliquotas"));
    }

    #[test]
    fn parse_json_array_body() {
        let body = r#"[
            {"codigoServico":"01.01.01.000","aliquota":2.0},
            {"codigo":"01.01.01.001","aliquota":3.5}
        ]"#;
        let rows = parse_aliquotas_json(body).expect("parse");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].codigo_trib_nac, "010101");
        assert_eq!(rows[0].codigo_trib_mun, "000");
        assert_eq!(rows[1].codigo_trib_mun, "001");
    }

    #[test]
    fn parse_json_wrapped_body() {
        let body = r#"{"aliquotas":[{"codigoServico":"01.02.01.000"}]}"#;
        let rows = parse_aliquotas_json(body).expect("parse");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].codigo_trib_nac, "010201");
    }

    #[test]
    fn parse_aliquotas_map_of_arrays() {
        let body = r#"{
            "mensagem": null,
            "aliquotas": {
                "01.01.01.000": [{"Aliq": 2.5, "DtIni": "2024-01-01T00:00:00"}],
                "01.01.01.001": [{"Aliq": 3.0, "DtIni": "2024-06-01"}]
            }
        }"#;
        let rows = parse_aliquotas_json(body).expect("parse");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].codigo_trib_mun, "000");
        assert_eq!(rows[1].codigo_trib_mun, "001");
    }

    fn adn_fixture(name: &str) -> String {
        std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/adn")
                .join(name),
        )
        .unwrap_or_else(|e| panic!("fixture adn/{name}: {e}"))
    }

    /// Corpo alinhado ao OpenAPI / saída de `./execute.sh adn` (sucesso).
    #[test]
    fn parse_catalog_sync_fixture_historico_010101() {
        let body = adn_fixture("historicoaliquotas_010101.json");
        let rows = parse_historico_aliquotas_body(&body).expect("parse");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].codigo_trib_nac, "010101");
        assert_eq!(rows[0].codigo_trib_mun, "000");
        assert!(rows[0].aliquota.is_some());
    }

    /// Resposta 404 / placeholder do script → sync municipal insere 0 linhas para o serviço.
    #[test]
    fn parse_catalog_sync_fixture_not_found_empty() {
        let body = adn_fixture("historicoaliquotas_not_found.json");
        let rows = parse_historico_aliquotas_body(&body).expect("parse");
        assert!(rows.is_empty());
    }

    /// Rio de Janeiro: CNA 010101 parametrizado como `01.01.01.001`, não `.000`.
    #[test]
    fn parse_rj_fixture_010101_desdobro_001() {
        let body = adn_fixture("historicoaliquotas_010101_001_rj.json");
        let rows = parse_historico_aliquotas_body(&body).expect("parse");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].codigo_trib_nac, "010101");
        assert_eq!(rows[0].codigo_trib_mun, "001");
    }

    #[test]
    fn convenio_fixture_from_execute_script() {
        let body = adn_fixture("convenio_3304557.json");
        let v: serde_json::Value = serde_json::from_str(&body).expect("json");
        assert!(v.get("parametrosConvenio").is_some());
        assert_eq!(
            v.get("mensagem").and_then(|m| m.as_str()),
            Some("Parâmetros do convênio recuperados com sucesso.")
        );
    }
}
