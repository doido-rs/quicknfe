//! Cliente HTTP ADN — parametrização municipal (público, sem mTLS).

mod codigo;

use doido::model::sea_orm::prelude::Decimal;
use std::str::FromStr;

use crate::services::api_error::ApiError;
use crate::settings;

pub use codigo::parse_codigo_parametrizacao;

#[derive(Debug, Clone)]
pub struct AliquotaParametrizacao {
    pub codigo_parametrizacao: String,
    pub codigo_trib_nac: String,
    pub codigo_trib_mun: String,
    pub aliquota: Option<Decimal>,
    pub vigencia_inicio: Option<chrono::NaiveDate>,
    pub vigencia_fim: Option<chrono::NaiveDate>,
}

/// Lista códigos administrados / alíquotas do município (IBGE 7 dígitos).
pub async fn get_aliquotas_municipio(codigo_ibge: &str) -> Result<Vec<AliquotaParametrizacao>, ApiError> {
    let base = settings::get().adn.base_url_normalized();
    let ibge = codigo_ibge.trim();
    let url = format!("{base}/parametros_municipais/{ibge}/aliquotas");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| ApiError::Internal(format!("cliente HTTP ADN: {e}")))?;

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

    if !status.is_success() {
        return Err(ApiError::Internal(format!(
            "ADN parametros_municipais HTTP {}: {}",
            status.as_u16(),
            truncate(&body, 500)
        )));
    }

    parse_aliquotas_json(&body)
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
    let items = extract_items(&value);
    let mut out = Vec::new();
    for item in items {
        if let Some(row) = map_item(&item) {
            out.push(row);
        }
    }
    Ok(out)
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

fn map_item(item: &serde_json::Value) -> Option<AliquotaParametrizacao> {
    let codigo_raw = item
        .get("codigoServico")
        .or_else(|| item.get("codigo"))
        .or_else(|| item.get("codServ"))
        .or_else(|| item.get("codigoParametrizacao"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())?;

    let (codigo_trib_nac, codigo_trib_mun) = parse_codigo_parametrizacao(codigo_raw)?;

    let aliquota = item
        .get("aliquota")
        .or_else(|| item.get("pAliq"))
        .and_then(parse_decimal);

    let vigencia_inicio = parse_date_field(item, &["dtIni", "dataInicio", "vigenciaInicio"]);
    let vigencia_fim = parse_date_field(item, &["dtFim", "dataFim", "vigenciaFim"]);

    Some(AliquotaParametrizacao {
        codigo_parametrizacao: codigo_raw.to_string(),
        codigo_trib_nac,
        codigo_trib_mun,
        aliquota,
        vigencia_inicio,
        vigencia_fim,
    })
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
}
