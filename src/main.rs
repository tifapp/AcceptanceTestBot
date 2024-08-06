mod generation;
mod git;
mod http;
mod language;
mod location;
mod operations;
mod slack;
mod tests_data;
mod utils;

use std::sync::Arc;

use anyhow::Result;
use dotenv::dotenv;
use http::{server::run_http_server, server_environment::ServerEnvironment};
use utils::log::bootstrap_logging;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv()?;
    bootstrap_logging();
    run_http_server(Arc::new(ServerEnvironment::current().await?)).await
}
