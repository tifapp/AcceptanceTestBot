use crate::{git::{metadata::RoswaalGitRepositoryMetadata, repo::{LibGit2RepositoryClient, RoswaalGitRepository}}, utils::sqlite::RoswaalSqlite};
use anyhow::Result;
use reqwest::Client;

/// A data type containing necessary structs for server operations.
pub struct ServerEnvironment {
    git_repository: RoswaalGitRepository<LibGit2RepositoryClient>,
    http_client: Client,
    sqlite: RoswaalSqlite,
    address: &'static str
}

impl ServerEnvironment {
    /// The production environment.
    pub async fn prod() -> Result<Self> {
        Ok(Self {
            git_repository: RoswaalGitRepository::open(
                &RoswaalGitRepositoryMetadata::for_tif_react_frontend()
            ).await?,
            http_client: Client::new(),
            sqlite: RoswaalSqlite::open("sqlite://roswaal.sqlite").await?,
            address: "TODO"
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
            address: "127.0.0.1:8082"
        })
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
