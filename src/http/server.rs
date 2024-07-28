use axum::{serve, Router};
#[cfg(test)]
use axum_test::TestServer;
use tokio::net::TcpListener;
use anyhow::Result;

use super::server_environment::ServerEnvironment;

/// Runs this tool as an http server using the specified `ServerEnvironment`.
pub async fn run_http_server(environment: &ServerEnvironment) -> Result<()> {
    let server = roswaal_server(&environment);
    let listener = TcpListener::bind(environment.address()).await?;
    Ok(serve(listener, server).await?)
}

/// Returns an http server for testing purposes.
#[cfg(test)]
pub fn test_http_server(environment: &ServerEnvironment) -> TestServer {
    TestServer::new(roswaal_server(environment)).unwrap()
}

fn roswaal_server(environment: &ServerEnvironment) -> Router<()> {
    Router::new()
}
