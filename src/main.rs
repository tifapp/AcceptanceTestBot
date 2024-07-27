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
use http::{routes::run_http_server, server_environment::ServerEnvironment};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv()?;
    let environment = ServerEnvironment::dev().await?;
    run_http_server(environment).await
}
