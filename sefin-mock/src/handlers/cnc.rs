use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct ConsultaCadQuery {
    #[serde(rename = "inscricaoFederal")]
    pub inscricao_federal: String,
    #[serde(rename = "codMunicipio")]
    pub cod_municipio: Option<i32>,
    #[serde(rename = "indicadorMunicipal")]
    pub _indicador_municipal: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct RespostaCad {
    lista_cadastro_municipal: Vec<CadastroMunicipal>,
    status_processamento: &'static str,
    alertas: Vec<()>,
    erros: Vec<()>,
    tipo_ambiente: &'static str,
    versao_aplicativo: &'static str,
    data_hora_processamento: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct CadastroMunicipal {
    origem_dados: &'static str,
    codigo_municipio: i32,
    tipo_ambiente: &'static str,
    id: i32,
    codigo_uf: i32,
    inf_cad: InfCad,
    dh_cadastro: String,
    dh_atualizacao: String,
    opcao_simples_nacional: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct InfCad {
    inscricao: String,
    tp_inscricao: &'static str,
    inscricao_municipal: String,
    data_inicio_inscricao_municipal: String,
    data_apenas_inicio_inscricao_municipal: String,
    razao_social: String,
    info_endereco: InfoEndereco,
    telefone: String,
    email: String,
    situacao_emissao_nfse: &'static str,
    situacao_cadastral: &'static str,
    motivo_situacao_cadastral: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct InfoEndereco {
    cep: String,
    logradouro: String,
    numero: String,
    complemento: String,
    bairro: String,
}

fn apenas_digitos(s: &str) -> String {
    s.chars().filter(|c| c.is_ascii_digit()).collect()
}

fn agora_iso() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn cadastro_demo(inscricao: &str, tp: &'static str, razao: &str, im: &str, cod_mun: i32) -> CadastroMunicipal {
    let now = agora_iso();
    let hoje = Utc::now().format("%Y-%m-%d").to_string();
    CadastroMunicipal {
        origem_dados: "CNC",
        codigo_municipio: cod_mun,
        tipo_ambiente: "HOMOLOGACAO",
        id: 1,
        codigo_uf: 35,
        inf_cad: InfCad {
            inscricao: inscricao.to_string(),
            tp_inscricao: tp,
            inscricao_municipal: im.to_string(),
            data_inicio_inscricao_municipal: now.clone(),
            data_apenas_inicio_inscricao_municipal: hoje,
            razao_social: razao.to_string(),
            info_endereco: InfoEndereco {
                cep: "01310100".into(),
                logradouro: "Av. Paulista".into(),
                numero: "1000".into(),
                complemento: "Sala 101".into(),
                bairro: "Bela Vista".into(),
            },
            telefone: "1130001000".into(),
            email: "fiscal@contribuinte-mock.nfse.local".into(),
            situacao_emissao_nfse: "HABILITADO",
            situacao_cadastral: "1",
            motivo_situacao_cadastral: "Mock sefin-mock",
        },
        dh_cadastro: now.clone(),
        dh_atualizacao: now,
        opcao_simples_nacional: "NENHUM",
    }
}

pub async fn get_cad(Query(q): Query<ConsultaCadQuery>) -> Response {
    let inscricao = apenas_digitos(&q.inscricao_federal);
    let cod_mun = q.cod_municipio.unwrap_or(3_550_308);

    let (status, lista) = match inscricao.as_str() {
        "99999999999999" => (
            "DOCUMENTOS_LOCALIZADOS",
            vec![
                cadastro_demo(
                    "99999999999999",
                    "CNPJ",
                    "Contribuinte Mock CNC LTDA",
                    "1234567",
                    cod_mun,
                ),
                cadastro_demo(
                    "99999999999999",
                    "CNPJ",
                    "Contribuinte Mock CNC LTDA",
                    "7654321",
                    cod_mun,
                ),
            ],
        ),
        "12345678901" => (
            "DOCUMENTOS_LOCALIZADOS",
            vec![cadastro_demo(
                "12345678901",
                "CPF",
                "Tomador Pessoa Física Mock",
                "9876543",
                cod_mun,
            )],
        ),
        "" => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "StatusProcessamento": "REJEICAO",
                    "Erros": [{"Codigo": "E001", "Descricao": "inscricaoFederal obrigatória"}],
                    "TipoAmbiente": "HOMOLOGACAO",
                    "DataHoraProcessamento": agora_iso(),
                })),
            )
                .into_response()
        }
        _ => (
            "NENHUM_DOCUMENTO_LOCALIZADO",
            Vec::new(),
        ),
    };

    let body = RespostaCad {
        lista_cadastro_municipal: lista,
        status_processamento: status,
        alertas: vec![],
        erros: vec![],
        tipo_ambiente: "HOMOLOGACAO",
        versao_aplicativo: "sefin-mock/0.1",
        data_hora_processamento: agora_iso(),
    };

    (StatusCode::OK, Json(body)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn mock_cnpj_retorna_cadastros() {
        let app = crate::build_app(crate::default_state());
        let res = app
            .oneshot(
                Request::builder()
                    .uri("/cnc/consulta/cad?inscricaoFederal=99999999999999")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }
}
