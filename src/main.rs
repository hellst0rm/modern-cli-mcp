// modern-cli-mcp/src/main.rs
// Test comment for pre-commit hook verification
mod tools;

use anyhow::Result;
use rmcp::{transport::stdio, ServiceExt};
use tools::ModernCliTools;
use tracing_subscriber::{self, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting Modern CLI Tools MCP server");

    let service = ModernCliTools::new()
        .serve(stdio())
        .await
        .inspect_err(|e| {
            tracing::error!("Server error: {:?}", e);
        })?;

    service.waiting().await?;
    Ok(())
}
