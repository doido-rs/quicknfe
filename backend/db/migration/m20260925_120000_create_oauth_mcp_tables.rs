use doido::model::migration::{create_table, drop_table};
use doido::model::sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260925_120000_create_oauth_mcp_tables"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_table(manager, "oauth_clients", |t| {
            t.string("client_id").not_null().unique_key();
            t.string("client_secret");
            t.text("redirect_uris").not_null();
            t.timestamp("created_at").not_null();
        })
        .await?;

        create_table(manager, "oauth_authorization_codes", |t| {
            t.string("code").not_null().unique_key();
            t.string("client_id").not_null();
            t.big_integer("user_id").not_null();
            t.string("code_challenge").not_null();
            t.string("redirect_uri").not_null();
            t.text("session_cookie").not_null();
            t.timestamp("expires_at").not_null();
            t.boolean("used").not_null().default(false);
        })
        .await?;

        create_table(manager, "oauth_access_tokens", |t| {
            t.string("jti").not_null().unique_key();
            t.big_integer("user_id").not_null();
            t.timestamp("expires_at").not_null();
            t.boolean("revoked").not_null().default(false);
            t.string("refresh_jti");
        })
        .await?;

        create_table(manager, "oauth_mcp_sessions", |t| {
            t.string("token_jti").not_null().unique_key();
            t.text("session_cookie").not_null();
            t.timestamp("updated_at").not_null();
        })
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        drop_table(manager, "oauth_mcp_sessions").await?;
        drop_table(manager, "oauth_access_tokens").await?;
        drop_table(manager, "oauth_authorization_codes").await?;
        drop_table(manager, "oauth_clients").await?;
        Ok(())
    }
}
