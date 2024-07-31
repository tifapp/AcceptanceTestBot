use std::env;

use crate::{git::{metadata::RoswaalGitRepositoryMetadata, repo::{LibGit2RepositoryClient, RoswaalGitRepository}}, utils::sqlite::RoswaalSqlite};
use anyhow::Result;
use log::info;
use reqwest::Client;

use super::password::EndpointPassword;

/// A data type containing necessary structs for server operations.
pub struct ServerEnvironment {
    git_repository: RoswaalGitRepository<LibGit2RepositoryClient>,
    http_client: Client,
    sqlite: RoswaalSqlite,
    address: &'static str,
    password: EndpointPassword
}

impl ServerEnvironment {
    /// The production environment.
    pub async fn prod() -> Result<Self> {
        Ok(Self {
            git_repository: RoswaalGitRepository::open(
                &RoswaalGitRepositoryMetadata::for_tif_react_frontend()
            ).await?,
            http_client: Client::new(),
            sqlite: RoswaalSqlite::open("./roswaal.sqlite").await?,
            address: "0.0.0.0:8080",
            password: EndpointPassword::prod()
        })
    }

    /// The development environment.
    pub async fn dev() -> Result<Self> {
        Ok(Self {
            git_repository: RoswaalGitRepository::open(
                &RoswaalGitRepositoryMetadata::for_testing()
            ).await?,
            http_client: Client::new(),
            sqlite: RoswaalSqlite::open("./roswaal-dev.sqlite").await?,
            address: "127.0.0.1:8082",
            password: EndpointPassword::dev()
        })
    }

    /// Returns the current environment.
    ///
    /// If the ROSWAAL_ENV environment variable is "dev", then the development environment is used.
    /// Otherwise, the production environment is used.
    pub async fn current() -> Result<Self> {
        if env::var("ROSWAAL_ENV").map(|d| d == "dev").is_ok() {
            info!("Using dev ServerEnvironment.");
            Self::dev().await
        } else {
            info!("Using production ServerEnvironment.");
            Self::prod().await
        }
    }
}

impl ServerEnvironment {
    pub fn git_repository(&self) -> &RoswaalGitRepository<LibGit2RepositoryClient> {
        &self.git_repository
    }

    pub fn http_client(&self) -> &Client {
        &self.http_client
    }

    pub fn sqlite(&self) -> &RoswaalSqlite {
        &self.sqlite
    }

    pub fn address(&self) -> &str {
        self.address
    }
}
