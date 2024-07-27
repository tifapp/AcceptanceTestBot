use axum::{serve, Router};
use tokio::net::TcpListener;
use anyhow::Result;

use super::server_environment::ServerEnvironment;

/// Runs this tool as an http server using the specified `ServerEnvironment`.
pub async fn run_http_server(environment: ServerEnvironment) -> Result<()> {
    let server: Router<()> = Router::new();
    let listener = TcpListener::bind(environment.address()).await?;
    Ok(serve(listener, server).await?)
}
