#[path = "../app/auth/mod.rs"]
mod auth;

#[path = "../app/settings/mod.rs"]
mod settings;

#[path = "../app/connectors/mod.rs"]
mod connectors;

#[path = "../app/services/mod.rs"]
mod services;

#[path = "../app/controllers/mod.rs"]
mod controllers;

#[path = "../app/models/mod.rs"]
mod models;

#[path = "../db/migration/mod.rs"]
mod migration;

#[path = "../db/seeds.rs"]
mod seed;

#[path = "../app/jobs/mod.rs"]
mod jobs;

#[path = "../app/mailers/mod.rs"]
mod mailers;

#[path = "../app/helpers/mod.rs"]
mod helpers;

#[path = "../app/generators/mod.rs"]
mod generators;

#[path = "../app/oauth/mod.rs"]
mod oauth;

#[path = "../app/mcp/mod.rs"]
mod mcp;

#[path = "../app/api_docs/mod.rs"]
mod api_docs;

#[path = "../config/routes.rs"]
mod routes;

mod startup;

fn should_bootstrap_nfse_catalog_on_boot() -> bool {
    matches!(
        std::env::args().nth(1).as_deref(),
        Some("server") | Some("worker")
    )
}

async fn bootstrap_nfse_catalog_jobs_on_boot() {
    if !should_bootstrap_nfse_catalog_on_boot() {
        return;
    }
    let Some(db) = doido_model::pool::try_pool().cloned() else {
        doido_core::tracing::warn!(
            "nfse catalog bootstrap: pool indisponível; rode migrate/seed ou verifique DATABASE_URL"
        );
        return;
    };
    if let Err(e) = services::nfse_catalog_bootstrap::bootstrap_nfse_catalog_jobs(&db).await {
        doido_core::tracing::warn!("nfse catalog bootstrap: {e}");
    }
}

#[tokio::main]
async fn main() {
    let _ = doido_model::pool::init().await;
    startup::prepare();
    settings::init();
    bootstrap_nfse_catalog_jobs_on_boot().await;
    // Delegates to the Doido CLI (server, console, db, worker, generate, …),
    // handing it this app's routes so `doido server` can boot the HTTP server.
    // The `jobs`/`mailers` modules above are compiled as part of this crate, so
    // generated jobs/mailers are type-checked even before they are wired up.
    //
    // `.migrator`/`.seeder` register the app's migration module (`db/migration/mod.rs`)
    // and seeder (`db/seeds.rs`) so `doido db migrate`/`doido db seed` run in-process
    // from this binary — no `cargo run` subprocess — and their SQL is logged.
    //
    // `doido generate generator <Name>` scaffolds an app generator under
    // `app/generators/` and registers it just above the marker below, so
    // `cargo doido generate <name>` can dispatch it alongside the built-ins.
    doido::Doido::new()
        .router(routes::router())
        .migrator::<migration::Migrator>()
        .seeder(seed::run)
        // @generated-generators
        .run()
        .await;
}
