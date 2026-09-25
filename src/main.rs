mod compiler;
mod mcp;
mod schema;
mod server;
mod template;

use server::Server;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Log to stderr so stdout remains clean for MCP JSON-RPC protocol
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "cv_writer_mcp=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    let server = Server::new()?;
    server.run_stdio().await?;
    Ok(())
}
