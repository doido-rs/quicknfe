use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NfsePostRequest {
    pub dps_xml_g_zip_b64: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NfsePostResponse {
    pub tipo_ambiente: i32,
    pub versao_aplicativo: String,
    pub data_hora_processamento: DateTime<Utc>,
    pub id_dps: String,
    pub chave_acesso: String,
    pub nfse_xml_g_zip_b64: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alertas: Option<Vec<MensagemProcessamento>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MensagemProcessamento {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mensagem: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codigo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub descricao: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complemento: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DpsConsultaResponse {
    pub id_dps: String,
    pub status: String,
    pub chave_acesso: String,
    pub data_hora_processamento: DateTime<Utc>,
    pub dps_xml_g_zip_b64: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nfse_xml_g_zip_b64: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StoredDps {
    pub id_dps: String,
    pub chave_acesso: String,
    pub dps_xml: String,
    pub nfse_xml: Option<String>,
    pub processado_em: DateTime<Utc>,
}
