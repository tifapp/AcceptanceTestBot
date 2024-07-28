mod language;
mod utils;
mod generation;
mod location;
mod git;
mod operations;
mod tests_data;
mod slack;
mod http;

use anyhow::Result;
use dotenv::dotenv;
use http::{server::run_http_server, server_environment::ServerEnvironment};
use utils::log::bootstrap_logging;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv()?;
    bootstrap_logging();
    run_http_server(&ServerEnvironment::current().await?).await
}
