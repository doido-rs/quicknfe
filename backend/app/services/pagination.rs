use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use doido::controller::Context;
use doido::model::sea_orm::{
    ConnectionTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Select,
};
use doido::model::sea_orm::ColumnTrait;
use serde::{Deserialize, Serialize};

use crate::services::api_error::ApiError;

#[derive(Debug, Serialize, Deserialize)]
struct CursorToken {
    id: i64,
}

#[derive(Debug, Clone)]
pub struct PageParams {
    pub after_id: Option<i64>,
    pub page_size: u64,
    pub q: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PageResponse<T> {
    pub items: Vec<T>,
    pub next_page_token: Option<String>,
}

pub const DEFAULT_PAGE_SIZE: u64 = 20;
pub const MAX_PAGE_SIZE: u64 = 100;

pub fn encode_page_token(id: i64) -> String {
    let payload = serde_json::to_string(&CursorToken { id }).expect("cursor json");
    URL_SAFE_NO_PAD.encode(payload.as_bytes())
}

pub fn decode_page_token(token: &str) -> Result<i64, ApiError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(token.trim())
        .map_err(|_| ApiError::Validation("page_token inválido".into()))?;
    let parsed: CursorToken = serde_json::from_slice(&bytes)
        .map_err(|_| ApiError::Validation("page_token inválido".into()))?;
    Ok(parsed.id)
}

#[derive(Debug, Deserialize)]
struct ListQueryParams {
    /// Query strings chegam como texto; `doido` pode expor JSON com string ou número.
    page_size: Option<serde_json::Value>,
    page_token: Option<String>,
    q: Option<String>,
}

pub fn parse_opt_i64_from_query(
    value: Option<serde_json::Value>,
    field: &str,
) -> Result<Option<i64>, ApiError> {
    let Some(value) = value else {
        return Ok(None);
    };
    match value {
        serde_json::Value::Null => Ok(None),
        serde_json::Value::Number(n) => n
            .as_i64()
            .ok_or_else(|| ApiError::Validation(format!("{field} inválido")))
            .map(Some),
        serde_json::Value::String(s) => {
            let s = s.trim();
            if s.is_empty() {
                Ok(None)
            } else {
                s.parse::<i64>()
                    .map(Some)
                    .map_err(|_| ApiError::Validation(format!("{field} inválido")))
            }
        }
        _ => Err(ApiError::Validation(format!("{field} inválido"))),
    }
}

fn parse_opt_u64_from_query(value: Option<serde_json::Value>, field: &str) -> Result<Option<u64>, ApiError> {
    let Some(value) = value else {
        return Ok(None);
    };
    match value {
        serde_json::Value::Null => Ok(None),
        serde_json::Value::Number(n) => n
            .as_u64()
            .ok_or_else(|| ApiError::Validation(format!("{field} inválido")))
            .map(Some),
        serde_json::Value::String(s) => {
            let s = s.trim();
            if s.is_empty() {
                Ok(None)
            } else {
                s.parse::<u64>()
                    .map(Some)
                    .map_err(|_| ApiError::Validation(format!("{field} inválido")))
            }
        }
        _ => Err(ApiError::Validation(format!("{field} inválido"))),
    }
}

pub fn page_params_from_ctx(ctx: &Context) -> Result<PageParams, ApiError> {
    let raw: ListQueryParams = ctx
        .query_params()
        .deserialize()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    let page_size = parse_opt_u64_from_query(raw.page_size, "page_size")?
        .unwrap_or(DEFAULT_PAGE_SIZE)
        .clamp(1, MAX_PAGE_SIZE);

    let after_id = match raw.page_token {
        Some(token) if !token.trim().is_empty() => Some(decode_page_token(token.trim())?),
        _ => None,
    };

    let q = raw
        .q
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    Ok(PageParams {
        after_id,
        page_size,
        q,
    })
}

pub async fn paginate_select<E, F, T, IdFn>(
    db: &impl ConnectionTrait,
    mut select: Select<E>,
    id_col: E::Column,
    params: PageParams,
    id_fn: IdFn,
    map: F,
) -> Result<PageResponse<T>, doido::model::sea_orm::DbErr>
where
    E: EntityTrait,
    F: Fn(E::Model) -> T,
    IdFn: Fn(&E::Model) -> i64,
{
    select = select.order_by_asc(id_col);
    if let Some(after_id) = params.after_id {
        select = select.filter(id_col.gt(after_id));
    }

    let limit = params.page_size.saturating_add(1);
    select = select.limit(limit);
    let rows = select.all(db).await?;

    let has_more = rows.len() as u64 > params.page_size;
    let page_rows: Vec<E::Model> = rows
        .into_iter()
        .take(params.page_size as usize)
        .collect();

    let next_page_token = if has_more {
        page_rows
            .last()
            .map(|row| encode_page_token(id_fn(row)))
    } else {
        None
    };

    Ok(PageResponse {
        items: page_rows.into_iter().map(map).collect(),
        next_page_token,
    })
}

#[cfg(test)]
mod tests {
    use super::parse_opt_u64_from_query;
    use crate::services::api_error::ApiError;

    #[test]
    fn page_size_accepts_query_string() {
        let n = parse_opt_u64_from_query(Some(serde_json::json!("20")), "page_size").unwrap();
        assert_eq!(n, Some(20));
    }

    #[test]
    fn page_size_accepts_json_number() {
        let n = parse_opt_u64_from_query(Some(serde_json::json!(20)), "page_size").unwrap();
        assert_eq!(n, Some(20));
    }

    #[test]
    fn page_size_rejects_invalid() {
        let err = parse_opt_u64_from_query(Some(serde_json::json!("abc")), "page_size").unwrap_err();
        assert!(matches!(err, ApiError::Validation(_)));
    }
}
