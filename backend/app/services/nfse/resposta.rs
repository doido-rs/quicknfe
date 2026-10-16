use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::compactacao::gzip_b64_para_xml;
use crate::services::api_error::ApiError;

/// Item de `erros` / `alertas` na resposta JSON da SEFIN Nacional.
/// O envelope usa camelCase; cada mensagem usa **PascalCase** (`Codigo`, `Descricao`, `Complemento`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MensagemProcessamento {
    #[serde(alias = "Codigo")]
    pub codigo: Option<String>,
    #[serde(alias = "Descricao")]
    pub descricao: Option<String>,
    #[serde(alias = "Complemento")]
    pub complemento: Option<String>,
    #[serde(alias = "Mensagem")]
    pub mensagem: Option<String>,
}

impl MensagemProcessamento {
    /// Campos alinhados a `logs_mensagens_retorno` (`codigo_erro`, `mensagem_retorno`, `correcao_sugerida`).
    pub fn campos_log(&self) -> (String, String, Option<String>) {
        let codigo = self
            .codigo
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "SEFIN".into());
        let mensagem = self
            .descricao
            .clone()
            .or_else(|| self.mensagem.clone())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "Erro retornado pela SEFIN".into());
        let correcao = self
            .complemento
            .clone()
            .filter(|s| !s.is_empty());
        (codigo, mensagem, correcao)
    }

    pub fn resumo(&self) -> String {
        let (codigo, mensagem, correcao) = self.campos_log();
        match correcao {
            Some(c) => format!("{codigo} — {mensagem} ({c})"),
            None => format!("{codigo} — {mensagem}"),
        }
    }
}

fn parse_sefin_data_hora(raw: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(raw)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NfsePostResponseSucesso {
    pub id_dps: Option<String>,
    pub chave_acesso: String,
    pub nfse_xml_g_zip_b64: String,
    pub alertas: Option<Vec<MensagemProcessamento>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NfsePostResponseErro {
    pub tipo_ambiente: Option<i32>,
    pub versao_aplicativo: Option<String>,
    pub data_hora_processamento: Option<String>,
    pub id_dps: Option<String>,
    #[serde(default)]
    pub erros: Vec<MensagemProcessamento>,
}

#[derive(Debug, Clone)]
pub struct ResultadoTransmissao {
    pub sucesso: bool,
    pub http_status: u16,
    pub id_dps: Option<String>,
    pub chave_acesso: Option<String>,
    pub nfse_xml: Option<String>,
    pub alertas: Vec<MensagemProcessamento>,
    pub erros: Vec<MensagemProcessamento>,
    pub data_hora_processamento: Option<DateTime<Utc>>,
    pub corpo_bruto: String,
}

pub fn interpretar_resposta_http(status: u16, body: &str) -> Result<ResultadoTransmissao, ApiError> {
    if status == 201 {
        let parsed: NfsePostResponseSucesso = serde_json::from_str(body).map_err(|e| {
            ApiError::Internal(format!("resposta 201 da SEFIN em formato inesperado: {e}"))
        })?;
        let nfse_xml = gzip_b64_para_xml(&parsed.nfse_xml_g_zip_b64)?;
        return Ok(ResultadoTransmissao {
            sucesso: true,
            http_status: status,
            id_dps: parsed.id_dps,
            chave_acesso: Some(parsed.chave_acesso),
            nfse_xml: Some(nfse_xml),
            alertas: parsed.alertas.unwrap_or_default(),
            erros: vec![],
            data_hora_processamento: None,
            corpo_bruto: body.to_string(),
        });
    }

    if let Ok(parsed) = serde_json::from_str::<NfsePostResponseErro>(body) {
        let data_hora_processamento = parsed
            .data_hora_processamento
            .as_deref()
            .and_then(parse_sefin_data_hora);
        return Ok(ResultadoTransmissao {
            sucesso: false,
            http_status: status,
            id_dps: parsed.id_dps,
            chave_acesso: None,
            nfse_xml: None,
            alertas: vec![],
            erros: parsed.erros,
            data_hora_processamento,
            corpo_bruto: body.to_string(),
        });
    }

    Ok(ResultadoTransmissao {
        sucesso: false,
        http_status: status,
        id_dps: None,
        chave_acesso: None,
        nfse_xml: None,
        alertas: vec![],
        erros: vec![MensagemProcessamento {
            codigo: Some(status.to_string()),
            descricao: Some("Resposta não JSON da SEFIN".into()),
            mensagem: Some(body.chars().take(500).collect()),
            complemento: None,
        }],
        data_hora_processamento: None,
        corpo_bruto: body.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_erro_sefin_e1235() {
        let body = r#"{"tipoAmbiente":2,"versaoAplicativo":"SefinNacional_1.6.0","dataHoraProcessamento":"2026-10-16T07:50:39.8993997-03:00","erros":[{"Codigo":"E1235","Descricao":"Falha no esquema XML do DF-e.","Complemento":"O namespace do elemento 'DPS' deve ser 'http://www.sped.fazenda.gov.br/nfse', mas foi encontrado 'http://www.gov.br/nfse'"}]}"#;
        let r = interpretar_resposta_http(400, body).unwrap();
        assert!(!r.sucesso);
        assert_eq!(r.erros.len(), 1);
        let (codigo, mensagem, correcao) = r.erros[0].campos_log();
        assert_eq!(codigo, "E1235");
        assert_eq!(mensagem, "Falha no esquema XML do DF-e.");
        assert!(correcao.as_deref().unwrap().contains("namespace"));
        assert!(r.data_hora_processamento.is_some());
    }

    #[test]
    fn parse_sucesso_201() {
        let xml = r#"<NFSe/>"#;
        let b64 = super::super::compactacao::xml_para_dps_gzip_b64(xml).unwrap();
        let body = format!(
            r#"{{"tipoAmbiente":2,"versaoAplicativo":"1","dataHoraProcessamento":"2025-01-01T00:00:00-03:00","idDps":"DPS1","chaveAcesso":"123","nfseXmlGZipB64":"{b64}"}}"#
        );
        let r = interpretar_resposta_http(201, &body).unwrap();
        assert!(r.sucesso);
        assert_eq!(r.chave_acesso.as_deref(), Some("123"));
        assert_eq!(r.nfse_xml.as_deref(), Some(xml));
    }
}
