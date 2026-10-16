use doido::controller::axum::body::Body;
use doido::controller::axum::http::{Request, StatusCode, header};
use doido::controller::axum::Router;
use tower::ServiceExt;

pub fn session_cookie_from_set_cookie(set_cookie: &str) -> String {
    set_cookie
        .split(';')
        .next()
        .unwrap_or(set_cookie)
        .to_string()
}

pub async fn sign_in_and_get_cookie(
    api_router: &Router,
    email: &str,
    password: &str,
) -> Result<String, String> {
    let body = serde_json::json!({ "email": email, "password": password }).to_string();
    let req = Request::builder()
        .method("POST")
        .uri("/users/sign_in")
        .header(header::ACCEPT, "application/json")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body))
        .map_err(|e| e.to_string())?;

    let resp = api_router
        .clone()
        .oneshot(req)
        .await
        .map_err(|e| e.to_string())?;
    if resp.status() != StatusCode::OK {
        return Err(format!("sign_in failed: {}", resp.status()));
    }
    let set_cookie = resp
        .headers()
        .get(header::SET_COOKIE)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| "missing session cookie".to_string())?;
    Ok(session_cookie_from_set_cookie(set_cookie))
}

pub async fn switch_company(
    api_router: &Router,
    session_cookie: &str,
    company_id: i64,
) -> Result<String, String> {
    let body = serde_json::json!({ "company_id": company_id }).to_string();
    let req = Request::builder()
        .method("POST")
        .uri("/tenant/company")
        .header(header::ACCEPT, "application/json")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::COOKIE, session_cookie)
        .body(Body::from(body))
        .map_err(|e| e.to_string())?;

    let resp = api_router
        .clone()
        .oneshot(req)
        .await
        .map_err(|e| e.to_string())?;
    if resp.status() != StatusCode::NO_CONTENT {
        return Err(format!("tenant switch failed: {}", resp.status()));
    }
    let set_cookie = resp
        .headers()
        .get(header::SET_COOKIE)
        .and_then(|v| v.to_str().ok())
        .map(|c| session_cookie_from_set_cookie(c))
        .unwrap_or_else(|| session_cookie.to_string());
    Ok(set_cookie)
}

pub async fn default_company_for_user(user_id: i64) -> Option<i64> {
    use doido::model::sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    use crate::models::company_usuario::{Column as MembroColumn, Entity as MembroEntity};

    let db = doido::model::pool::pool();
    MembroEntity::find()
        .filter(MembroColumn::UserId.eq(user_id))
        .one(db)
        .await
        .ok()
        .flatten()
        .map(|m| m.company_id)
}
