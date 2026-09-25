tokio::task_local! {
    pub static MCP_TOKEN_JTI: String;
}

pub async fn with_token_jti<F, T>(jti: String, f: F) -> T
where
    F: std::future::Future<Output = T>,
{
    MCP_TOKEN_JTI.scope(jti, f).await
}
