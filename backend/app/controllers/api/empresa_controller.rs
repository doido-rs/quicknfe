use doido::controller::controller;
use doido::model::sea_orm::{ActiveModelTrait, Set};
use serde::Deserialize;

use crate::auth::tenant;
use crate::models::empresa::ActiveModel as EmpresaActive;
use crate::services::api_error::{ApiErrExt, ApiError};
use crate::services::dto::EmpresaPublic;

#[derive(Debug, Deserialize)]
pub struct EmpresaForm {
    pub cpf_cnpj: Option<String>,
    pub razao_social: Option<String>,
    pub inscricao_municipal: Option<String>,
    pub inscricao_estadual: Option<String>,
    pub regime_tributario: Option<i32>,
    pub codigo_municipio_ibge: Option<String>,
    pub uf: Option<String>,
}

pub struct ApiEmpresaController;

#[controller]
impl ApiEmpresaController {
    pub async fn show(
        mut ctx: doido::controller::Context,
    ) -> doido::controller::Response {
        let access = tenant::tenant_from_ctx(ctx).await.api()?;
        ctx.json(EmpresaPublic::from(access.empresa))
    }

    pub async fn update(
        mut ctx: doido::controller::Context,
    ) -> doido::controller::Response {
        let access = tenant::tenant_from_ctx(ctx).await.api()?;
        let form: EmpresaForm = ctx
            .body_json()
            .await
            .map_err(|e| ApiError::Validation(e.to_string())).api()?;
        let mut record: EmpresaActive = access.empresa.into();
        if let Some(v) = form.cpf_cnpj {
            record.cpf_cnpj = Set(v);
        }
        if let Some(v) = form.razao_social {
            record.razao_social = Set(v);
        }
        if let Some(v) = form.inscricao_municipal {
            record.inscricao_municipal = Set(Some(v));
        }
        if let Some(v) = form.inscricao_estadual {
            record.inscricao_estadual = Set(Some(v));
        }
        if let Some(v) = form.regime_tributario {
            record.regime_tributario = Set(v);
        }
        if let Some(v) = form.codigo_municipio_ibge {
            record.codigo_municipio_ibge = Set(v);
        }
        if let Some(v) = form.uf {
            record.uf = Set(v);
        }
        let updated = record
            .update(ctx.db())
            .await
            .map_err(|e| ApiError::Internal(e.to_string())).api()?;
        ctx.json(EmpresaPublic::from(updated))
    }
}
