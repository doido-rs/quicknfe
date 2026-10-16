use super::tipos::{ConsultaCadParams, RespostaConsultaCad};
use crate::connectors::cnc;
use crate::connectors::mtls::CredenciaisMtls;
use crate::services::api_error::ApiError;
use crate::settings;

fn apenas_digitos(s: &str) -> String {
    s.chars().filter(|c| c.is_ascii_digit()).collect()
}

pub async fn consulta_cad(
    params: ConsultaCadParams,
    credenciais: Option<CredenciaisMtls>,
) -> Result<RespostaConsultaCad, ApiError> {
    let inscricao = apenas_digitos(&params.inscricao_federal);
    if inscricao.len() != 11 && inscricao.len() != 14 {
        return Err(ApiError::Validation(
            "inscricaoFederal deve ser CPF (11 dígitos) ou CNPJ (14 dígitos)".into(),
        ));
    }

    let base = settings::get().cnc.base_url_normalized();
    let mut query = format!(
        "inscricaoFederal={}",
        urlencoding::encode(&inscricao)
    );
    if let Some(cod) = params.cod_municipio {
        query.push_str(&format!("&codMunicipio={cod}"));
    }
    if let Some(ref ind) = params.indicador_municipal {
        if !ind.trim().is_empty() {
            query.push_str(&format!(
                "&indicadorMunicipal={}",
                urlencoding::encode(ind.trim())
            ));
        }
    }
    let url = format!("{base}/cad?{query}");

    let (status, body) = cnc::get_cad(&url, &base, credenciais).await?;

    if status != 404 && !(200..300).contains(&status) {
        return Err(ApiError::Validation(format!(
            "CNC retornou HTTP {status}: {}",
            body.chars().take(500).collect::<String>()
        )));
    }

    serde_json::from_str::<RespostaConsultaCad>(&body).map_err(|e| {
        ApiError::Internal(format!(
            "resposta CNC não é JSON válido (HTTP {status}): {e}; corpo: {}",
            body.chars().take(200).collect::<String>()
        ))
    })
}
