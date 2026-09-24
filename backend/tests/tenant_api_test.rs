//! API multi-tenant — requer MySQL de teste (`DOIDO_ENV=test`, migrations aplicadas).

#![allow(dead_code, unused_imports)]

#[path = "../app/auth/mod.rs"]
mod auth;
#[path = "../app/controllers/mod.rs"]
mod controllers;
#[path = "../app/models/mod.rs"]
mod models;
#[path = "../app/services/mod.rs"]
mod services;
#[path = "../app/helpers/mod.rs"]
mod helpers;
#[path = "../config/routes.rs"]
mod routes;

use chrono::Utc;
use doido::controller::axum::body::Body;
use doido::controller::axum::http::{Request, StatusCode};
use doido::controller::stack::MiddlewareStack;
use doido::model::password::hash_password;
use doido::model::sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use doido::controller::axum::http::header;
use models::cliente::{Column as ClienteColumn, Entity as ClienteEntity};
use models::empresa::ActiveModel as EmpresaActive;
use models::empresa_usuario::ActiveModel as MembroActive;
use models::user::ActiveModel as UserActive;
use tower::ServiceExt;

fn test_router() -> doido::controller::axum::Router {
    let config = doido::controller::config::load();
    MiddlewareStack::default()
        .with_cors_config(config.middleware().cors.clone())
        .apply(routes::router())
}

async fn setup_runtime() -> std::sync::MutexGuard<'static, ()> {
    std::env::set_var("DOIDO_ENV", "test");
    let guard = doido::model::pool::test_lock();
    if doido::model::pool::try_pool().is_none() {
        doido::model::pool::init()
            .await
            .expect("connect test database — run migrations first");
    }
    let config = doido_auth::load_config();
    doido_auth::init(doido::model::pool::pool().clone(), &config)
        .await
        .expect("auth init");
    guard
}

async fn send_json(
    app: &doido::controller::axum::Router,
    method: &str,
    uri: &str,
    body: &str,
    cookie: Option<&str>,
) -> (StatusCode, String, Option<String>) {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::ACCEPT, "application/json");
    if !body.is_empty() {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
    }
    if let Some(c) = cookie {
        builder = builder.header(header::COOKIE, c);
    }
    let req = builder.body(Body::from(body.to_string())).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let set_cookie = resp
        .headers()
        .get(header::SET_COOKIE)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let bytes = doido::controller::axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    (status, String::from_utf8_lossy(&bytes).to_string(), set_cookie)
}

fn session_cookie(set_cookie: &str) -> String {
    set_cookie
        .split(';')
        .next()
        .unwrap_or(set_cookie)
        .to_string()
}

async fn sign_in(app: &doido::controller::axum::Router, email: &str, password: &str) -> String {
    let (status, _, cookie) = send_json(
        app,
        "POST",
        "/users/sign_in",
        &format!(r#"{{"email":"{email}","password":"{password}"}}"#),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "sign_in failed");
    session_cookie(cookie.as_ref().expect("session cookie"))
}

struct TenantFixture {
    user_email: String,
    password: String,
    empresa_id: i64,
    other_empresa_id: i64,
    other_cliente_id: i64,
}

async fn seed_two_tenant_fixture() -> TenantFixture {
    let db = doido::model::pool::pool();
    let suffix = uuid_simple();
    let email_a = format!("tenant_a_{suffix}@test.local");
    let email_b = format!("tenant_b_{suffix}@test.local");
    let password = "TestPass123!";

    let user_a = UserActive {
        email: Set(email_a.clone()),
        password_digest: Set(hash_password(password).unwrap()),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        ..Default::default()
    }
    .insert(db)
    .await
    .unwrap();

    let user_b = UserActive {
        email: Set(email_b.clone()),
        password_digest: Set(hash_password(password).unwrap()),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        ..Default::default()
    }
    .insert(db)
    .await
    .unwrap();

    let emp_a = EmpresaActive {
        cpf_cnpj: Set(unique_cnpj(&suffix, 1)),
        razao_social: Set(format!("Empresa A {suffix}")),
        regime_tributario: Set(1),
        codigo_municipio_ibge: Set("3550308".into()),
        uf: Set("SP".into()),
        criado_em: Set(Utc::now()),
        ..Default::default()
    }
    .insert(db)
    .await
    .unwrap();

    let emp_b = EmpresaActive {
        cpf_cnpj: Set(unique_cnpj(&suffix, 2)),
        razao_social: Set(format!("Empresa B {suffix}")),
        regime_tributario: Set(1),
        codigo_municipio_ibge: Set("3550308".into()),
        uf: Set("SP".into()),
        criado_em: Set(Utc::now()),
        ..Default::default()
    }
    .insert(db)
    .await
    .unwrap();

    MembroActive {
        user_id: Set(user_a.id),
        empresa_id: Set(emp_a.id),
        papel: Set("proprietario".into()),
        ..Default::default()
    }
    .insert(db)
    .await
    .unwrap();

    MembroActive {
        user_id: Set(user_b.id),
        empresa_id: Set(emp_b.id),
        papel: Set("proprietario".into()),
        ..Default::default()
    }
    .insert(db)
    .await
    .unwrap();

    let other_cliente = models::cliente::ActiveModel {
        empresa_id: Set(emp_b.id),
        cpf_cnpj: Set("99888777000166".into()),
        razao_social: Set("Cliente B".into()),
        ..Default::default()
    }
    .insert(db)
    .await
    .unwrap();

    TenantFixture {
        user_email: email_a,
        password: password.into(),
        empresa_id: emp_a.id,
        other_empresa_id: emp_b.id,
        other_cliente_id: other_cliente.id,
    }
}

fn uuid_simple() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};
    static SEQ: AtomicU64 = AtomicU64::new(1);
    let n = SEQ.fetch_add(1, Ordering::SeqCst);
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{n}_{t}")
}

fn unique_cnpj(suffix: &str, salt: u8) -> String {
    let mut n: u32 = salt as u32;
    for b in suffix.as_bytes() {
        n = n.wrapping_mul(31).wrapping_add(*b as u32);
    }
    let prefix = if salt == 1 { "11222333" } else { "22333444" };
    format!("{prefix}{:06}", n % 1_000_000)
}

#[tokio::test]
async fn login_me_and_tenant_cliente_flow() {
    let _guard = setup_runtime().await;
    let app = test_router();
    let fx = seed_two_tenant_fixture().await;

    let cookie = sign_in(&app, &fx.user_email, &fx.password).await;

    let (status, body, _) = send_json(&app, "GET", "/api/me", "", Some(&cookie)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains(&fx.user_email));

    let (status, _, _) = send_json(
        &app,
        "GET",
        "/api/clientes",
        "",
        Some(&cookie),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, _, switch_cookie) = send_json(
        &app,
        "POST",
        "/tenant/empresa",
        &format!(r#"{{"empresa_id":{}}}"#, fx.empresa_id),
        Some(&cookie),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let session = switch_cookie
        .as_ref()
        .map(|c| session_cookie(c))
        .unwrap_or(cookie);

    let (status, body, _) = send_json(&app, "GET", "/api/me", "", Some(&session)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains("\"empresa_ativa\"") && !body.contains("\"empresa_ativa\":null"),
        "me response: {body}"
    );

    let (status, body, _) = send_json(
        &app,
        "POST",
        "/api/clientes",
        r#"{"cpf_cnpj":"12345678901","razao_social":"Cliente Teste"}"#,
        Some(&session),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "create cliente body: {body}");
    assert!(body.contains("Cliente Teste"));

    let rows = ClienteEntity::find()
        .filter(ClienteColumn::EmpresaId.eq(fx.empresa_id))
        .all(doido::model::pool::pool())
        .await
        .unwrap();
    assert!(!rows.is_empty());
}

#[tokio::test]
async fn tenant_isolation_on_cliente_patch() {
    let _guard = setup_runtime().await;
    let app = test_router();
    let fx = seed_two_tenant_fixture().await;
    let cookie = sign_in(&app, &fx.user_email, &fx.password).await;
    let (_, _, switch_cookie) = send_json(
        &app,
        "POST",
        "/tenant/empresa",
        &format!(r#"{{"empresa_id":{}}}"#, fx.empresa_id),
        Some(&cookie),
    )
    .await;
    let session = switch_cookie
        .as_ref()
        .map(|c| session_cookie(c))
        .unwrap_or(cookie);

    let (status, _, _) = send_json(
        &app,
        "PATCH",
        &format!("/api/clientes/{}", fx.other_cliente_id),
        r#"{"cpf_cnpj":"99888777000166","razao_social":"Hacked","inscricao_municipal":null,"inscricao_estadual":null,"logradouro":null,"numero":null,"bairro":null,"codigo_municipio_ibge":null,"uf":null,"email":null,"telefone":null}"#,
        Some(&session),
    )
    .await;
    assert!(
        status == StatusCode::NOT_FOUND || status == StatusCode::FORBIDDEN,
        "cross-tenant patch must not succeed, got {status}"
    );
}
