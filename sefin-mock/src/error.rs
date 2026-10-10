use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MensagemErro {
    pub codigo: String,
    pub descricao: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mensagem: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complemento: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErroBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_dps: Option<String>,
    pub erros: Vec<MensagemErro>,
}

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    body: ErroBody,
}

impl ApiError {
    pub fn bad_request(descricao: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            body: ErroBody {
                id_dps: None,
                erros: vec![MensagemErro {
                    codigo: "E001".into(),
                    descricao: descricao.into(),
                    mensagem: None,
                    complemento: None,
                }],
            },
        }
    }

    pub fn not_found(id: &str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            body: ErroBody {
                id_dps: Some(id.to_string()),
                erros: vec![MensagemErro {
                    codigo: "E404".into(),
                    descricao: "DPS não encontrada".into(),
                    mensagem: Some(format!("id={id}")),
                    complemento: None,
                }],
            },
        }
    }

    pub fn internal(descricao: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: ErroBody {
                id_dps: None,
                erros: vec![MensagemErro {
                    codigo: "E500".into(),
                    descricao: descricao.into(),
                    mensagem: None,
                    complemento: None,
                }],
            },
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(self.body)).into_response()
    }
}
