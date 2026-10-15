use doido::controller::controller;
use doido::model::sea_orm::{ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, Set};
use serde::Deserialize;

use crate::models::cliente::{
    ActiveModel as ClienteActive, Column as ClienteColumn, Entity as ClienteEntity, Model as Cliente,
};
use crate::services::api_error::{db, id_param, json_body, require_tenant, scoped, to_response};
use crate::services::dto::ClienteDto;
use crate::services::municipio::{map_by_ids, require_municipio};
use crate::services::pagination::{page_params_from_ctx, paginate_select, PageResponse};
use crate::services::tenant_scope;

#[derive(Debug, Deserialize)]
pub struct ClienteForm {
    pub cpf_cnpj: String,
    pub razao_social: String,
    pub inscricao_municipal: Option<String>,
    pub inscricao_estadual: Option<String>,
    pub logradouro: Option<String>,
    pub numero: Option<String>,
    pub bairro: Option<String>,
    pub cep: Option<String>,
    pub municipio_id: Option<i64>,
    pub email: Option<String>,
    pub telefone: Option<String>,
}

async fn cliente_public(
    db: &impl doido::model::sea_orm::ConnectionTrait,
    cliente: Cliente,
) -> Result<ClienteDto, crate::services::api_error::ApiError> {
    let municipio = match cliente.municipio_id {
        Some(id) => Some(require_municipio(db, id).await?),
        None => None,
    };
    Ok(ClienteDto::from_parts(
        cliente,
        municipio.as_ref(),
    ))
}

async fn clientes_public(
    db: &impl doido::model::sea_orm::ConnectionTrait,
    clientes: Vec<Cliente>,
) -> Result<Vec<ClienteDto>, crate::services::api_error::ApiError> {
    let municipios = map_by_ids(
        db,
        clientes.iter().filter_map(|c| c.municipio_id),
    )
    .await?;
    Ok(clientes
        .into_iter()
        .map(|c| {
            let municipio = c
                .municipio_id
                .and_then(|id| municipios.get(&id));
            ClienteDto::from_parts(c, municipio)
        })
        .collect())
}

pub struct ApiClientesController;

#[controller]
impl ApiClientesController {
    pub async fn index(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let prestador_id = match crate::api!(to_response(tenant_scope::empresa_id_from_list_query(&ctx)))
        {
            Some(id) => {
                crate::api!(scoped(
                    tenant_scope::prestador_for_company(ctx.db(), access.company.id, id).await,
                ));
                id
            }
            None => access.prestador.id,
        };
        let params = crate::api!(to_response(page_params_from_ctx(&ctx)));
        let mut select = ClienteEntity::find().filter(ClienteColumn::EmpresaId.eq(prestador_id));
        if let Some(ref q) = params.q {
            select = select.filter(
                Condition::any()
                    .add(ClienteColumn::RazaoSocial.contains(q))
                    .add(ClienteColumn::CpfCnpj.contains(q)),
            );
        }
        let page = crate::api!(db(
            paginate_select(
                ctx.db(),
                select,
                ClienteColumn::Id,
                params,
                |m| m.id,
                |m| m,
            )
            .await,
        ));
        let items = crate::api!(to_response(clientes_public(ctx.db(), page.items).await));
        ctx.json(PageResponse {
            items,
            next_page_token: page.next_page_token,
        })
    }

    pub async fn show(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let id = crate::api!(id_param(&ctx, "id"));
        let row = crate::api!(scoped(tenant_scope::cliente_for_tenant(ctx.db(), access.prestador.id, id).await));
        let dto = crate::api!(to_response(cliente_public(ctx.db(), row).await));
        ctx.json(dto)
    }

    pub async fn create(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let form: ClienteForm = crate::api!(json_body(ctx).await);
        if let Some(municipio_id) = form.municipio_id {
            crate::api!(to_response(require_municipio(ctx.db(), municipio_id).await));
        }
        let created = crate::api!(db(
            ClienteActive {
                empresa_id: Set(access.prestador.id),
                cpf_cnpj: Set(form.cpf_cnpj),
                razao_social: Set(form.razao_social),
                inscricao_municipal: Set(form.inscricao_municipal),
                inscricao_estadual: Set(form.inscricao_estadual),
                logradouro: Set(form.logradouro),
                numero: Set(form.numero),
                bairro: Set(form.bairro),
                cep: Set(form.cep),
                municipio_id: Set(form.municipio_id),
                email: Set(form.email),
                telefone: Set(form.telefone),
                ..Default::default()
            }
            .insert(ctx.db())
            .await,
        ));
        let dto = crate::api!(to_response(cliente_public(ctx.db(), created).await));
        ctx.json(dto)
    }

    pub async fn update(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let id = crate::api!(id_param(&ctx, "id"));
        let existing = crate::api!(scoped(tenant_scope::cliente_for_tenant(ctx.db(), access.prestador.id, id).await));
        let form: ClienteForm = crate::api!(json_body(ctx).await);
        if let Some(municipio_id) = form.municipio_id {
            crate::api!(to_response(require_municipio(ctx.db(), municipio_id).await));
        }
        let mut record: ClienteActive = existing.into();
        record.cpf_cnpj = Set(form.cpf_cnpj);
        record.razao_social = Set(form.razao_social);
        record.inscricao_municipal = Set(form.inscricao_municipal);
        record.inscricao_estadual = Set(form.inscricao_estadual);
        record.logradouro = Set(form.logradouro);
        record.numero = Set(form.numero);
        record.bairro = Set(form.bairro);
        record.cep = Set(form.cep);
        record.municipio_id = Set(form.municipio_id);
        record.email = Set(form.email);
        record.telefone = Set(form.telefone);
        let updated = crate::api!(db(record.update(ctx.db()).await));
        let dto = crate::api!(to_response(cliente_public(ctx.db(), updated).await));
        ctx.json(dto)
    }

    pub async fn destroy(mut ctx: doido::controller::Context) -> doido::controller::Response {
        let access = crate::api!(require_tenant(ctx).await);
        let id = crate::api!(id_param(&ctx, "id"));
        crate::api!(scoped(tenant_scope::cliente_for_tenant(ctx.db(), access.prestador.id, id).await));
        crate::api!(db(ClienteEntity::delete_by_id(id).exec(ctx.db()).await));
        ctx.status(204)
    }
}
