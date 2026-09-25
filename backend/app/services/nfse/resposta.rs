use serde::{Deserialize, Serialize};

use super::compactacao::gzip_b64_para_xml;
use crate::services::api_error::ApiError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MensagemProcessamento {
    pub mensagem: Option<String>,
    pub codigo: Option<String>,
    pub descricao: Option<String>,
    pub complemento: Option<String>,
}

impl MensagemProcessamento {
    pub fn resumo(&self) -> String {
        [
            self.codigo.as_deref(),
            self.descricao.as_deref(),
            self.mensagem.as_deref(),
            self.complemento.as_deref(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" — ")
    }
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
    pub id_dps: Option<String>,
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
            corpo_bruto: body.to_string(),
        });
    }

    if let Ok(parsed) = serde_json::from_str::<NfsePostResponseErro>(body) {
        return Ok(ResultadoTransmissao {
            sucesso: false,
            http_status: status,
            id_dps: parsed.id_dps,
            chave_acesso: None,
            nfse_xml: None,
            alertas: vec![],
            erros: parsed.erros,
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
        corpo_bruto: body.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
