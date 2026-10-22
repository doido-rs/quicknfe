use doido::controller::controller;
use doido::model::sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde::Deserialize;

use crate::models::certificado_digital::{Column as CertColumn, Entity as CertEntity};
use crate::services::api_error::{require_tenant, scoped, to_response, ApiError};
use crate::services::cnc::{
    consulta_cad, gravar_op_simp_nac_verificado, resposta_publica, ConsultaCadParams,
};
use crate::services::dto::decode_certificado_senha;
use crate::services::nfse::CredenciaisMtls;
use crate::services::tenant_scope;

#[derive(Debug, Deserialize)]
struct ConsultaCadQuery {
    inscricao_federal: String,
    cod_municipio: Option<i32>,
    indicador_municipal: Option<String>,
    empresa_id: Option<i64>,
    certificado_id: Option<i64>,
}

pub struct ApiCncController;

async fn credenciais_mtls(
    db: &doido::model::sea_orm::DatabaseConnection,
    empresa_id: i64,
    certificado_id: Option<i64>,
) -> Result<Option<CredenciaisMtls>, ApiError> {
    let base = crate::settings::get().cnc.base_url_normalized();
    let lower = base.to_ascii_lowercase();
    if lower.contains("127.0.0.1") || lower.contains("localhost") {
        return Ok(None);
    }

    let cert = if let Some(id) = certificado_id {
        tenant_scope::certificado_for_tenant(db, empresa_id, id).await?
    } else {
        CertEntity::find()
            .filter(CertColumn::EmpresaId.eq(empresa_id))
            .order_by_desc(CertColumn::CriadoEm)
            .one(db)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?
            .ok_or_else(|| {
                ApiError::Validation(
                    "nenhum certificado digital cadastrado para o prestador (necessário para CNC)".into(),
                )
            })?
    };

    let caminho = cert.caminho_arquivo.ok_or_else(|| {
        ApiError::Validation("certificado sem caminho_arquivo configurado".into())
    })?;

    Ok(Some(CredenciaisMtls {
        caminho_pfx: caminho,
        senha_pfx: decode_certificado_senha(&cert.senha_encriptada),
    }))
}

#[controller]
impl ApiCncController {
    /// `GET /api/cnc/cad` — proxy para consulta CNC por inscrição federal (CPF/CNPJ).
    pub async fn consulta_cad(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let q: ConsultaCadQuery = crate::api!(to_response(
            ctx.query_params()
                .deserialize()
                .map_err(|e| ApiError::Validation(e.to_string())),
        ));

        let empresa_id = q.empresa_id.unwrap_or(access.prestador.id);
        crate::api!(scoped(
            tenant_scope::prestador_for_company(ctx.db(), access.company.id, empresa_id).await,
        ));

        let credenciais =
            crate::api!(to_response(credenciais_mtls(ctx.db(), empresa_id, q.certificado_id).await));

        let raw = crate::api!(to_response(
            consulta_cad(
                ConsultaCadParams {
                    inscricao_federal: q.inscricao_federal,
                    cod_municipio: q.cod_municipio,
                    indicador_municipal: q.indicador_municipal,
                },
                credenciais,
            )
            .await,
        ));

        crate::api!(to_response(
            gravar_op_simp_nac_verificado(ctx.db(), empresa_id, &raw).await,
        ));

        let public = crate::api!(to_response(resposta_publica(ctx.db(), raw).await));
        ctx.json(public)
    }
}
