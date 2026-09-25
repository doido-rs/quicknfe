use std::sync::OnceLock;

static CONFIG: OnceLock<McpConfig> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct McpConfig {
    pub enabled: bool,
    pub public_url: String,
    pub oauth_signing_secret: String,
    pub access_token_ttl_secs: u64,
    pub refresh_token_ttl_secs: u64,
}

impl McpConfig {
    pub fn init() -> &'static McpConfig {
        CONFIG.get_or_init(|| {
            let enabled = std::env::var("MCP_ENABLED")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or_else(|_| {
                    std::env::var("DOIDO_ENV")
                        .map(|e| e == "development")
                        .unwrap_or(false)
                });

            let public_url = std::env::var("MCP_PUBLIC_URL")
                .unwrap_or_else(|_| "http://localhost:3000".into());

            let oauth_signing_secret = std::env::var("MCP_OAUTH_SIGNING_SECRET").unwrap_or_else(
                |_| {
                    if enabled {
                        eprintln!(
                            "warning: MCP_OAUTH_SIGNING_SECRET not set; using insecure dev default"
                        );
                    }
                    "quicknfe-mcp-dev-signing-secret-change-me".into()
                },
            );

            let access_token_ttl_secs = std::env::var("MCP_ACCESS_TOKEN_TTL_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3600);

            let refresh_token_ttl_secs = std::env::var("MCP_REFRESH_TOKEN_TTL_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(604_800);

            McpConfig {
                enabled,
                public_url: public_url.trim_end_matches('/').to_string(),
                oauth_signing_secret,
                access_token_ttl_secs,
                refresh_token_ttl_secs,
            }
        })
    }

    pub fn get() -> &'static McpConfig {
        CONFIG
            .get()
            .expect("McpConfig::init() must run before use")
    }

    pub fn resource_url(&self) -> String {
        format!("{}/mcp", self.public_url)
    }

    pub fn prm_url(&self) -> String {
        format!("{}/.well-known/oauth-protected-resource", self.public_url)
    }

    pub fn issuer(&self) -> String {
        self.public_url.clone()
    }
}

pub fn enabled() -> bool {
    McpConfig::init().enabled
}
