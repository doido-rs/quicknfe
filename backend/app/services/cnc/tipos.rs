use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct ConsultaCadParams {
    pub inscricao_federal: String,
    pub cod_municipio: Option<i32>,
    pub indicador_municipal: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RespostaConsultaCad {
    #[serde(default)]
    pub lista_cadastro_municipal: Vec<CadastroMunicipalCnc>,
    pub status_processamento: Option<String>,
    #[serde(default)]
    pub alertas: Vec<MensagemCnc>,
    #[serde(default)]
    pub erros: Vec<MensagemCnc>,
    pub tipo_ambiente: Option<String>,
    pub versao_aplicativo: Option<String>,
    pub data_hora_processamento: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CadastroMunicipalCnc {
    pub codigo_municipio: Option<i64>,
    pub inf_cad: Option<InfCadCnc>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct InfCadCnc {
    pub inscricao: Option<String>,
    pub tp_inscricao: Option<String>,
    pub inscricao_municipal: Option<String>,
    pub razao_social: Option<String>,
    pub telefone: Option<String>,
    pub email: Option<String>,
    pub info_endereco: Option<InfoEnderecoCnc>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct InfoEnderecoCnc {
    pub cep: Option<String>,
    pub logradouro: Option<String>,
    pub numero: Option<String>,
    pub complemento: Option<String>,
    pub bairro: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MensagemCnc {
    pub codigo: Option<String>,
    pub descricao: Option<String>,
}
