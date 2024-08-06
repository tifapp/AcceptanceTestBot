use std::{env, sync::Arc};

use crate::{
    git::{
        metadata::RoswaalGitRepositoryMetadata,
        pull_request::GithubPullRequestOpen,
        repo::{LibGit2RepositoryClient, RoswaalGitRepository},
    },
    slack::message::SlackSendMessage,
    utils::{env::RoswaalEnvironement, sqlite::RoswaalSqlite},
};
use anyhow::Result;
use log::info;
use reqwest::Client;

use super::password::EndpointPassword;

/// A data type containing necessary structs for server operations.
pub struct ServerEnvironment {
    git_repository: RoswaalGitRepository<LibGit2RepositoryClient>,
    http_client: Arc<Client>,
    sqlite: Arc<RoswaalSqlite>,
    address: &'static str,
    password: EndpointPassword,
}

impl ServerEnvironment {
    /// The production environment.
    pub async fn prod() -> Result<Self> {
        Ok(Self {
            git_repository: RoswaalGitRepository::open(
                &RoswaalGitRepositoryMetadata::for_tif_react_frontend(),
            )
            .await?,
            http_client: Arc::new(Client::new()),
            sqlite: Arc::new(RoswaalSqlite::open("./roswaal.sqlite").await?),
            address: "0.0.0.0:8080",
            password: EndpointPassword::prod(),
        })
    }

    /// The development environment.
    pub async fn dev() -> Result<Self> {
        Ok(Self {
            git_repository:
                RoswaalGitRepository::open(&RoswaalGitRepositoryMetadata::for_testing()).await?,
            http_client: Arc::new(Client::new()),
            sqlite: Arc::new(RoswaalSqlite::open("./roswaal-dev.sqlite").await?),
            address: "127.0.0.1:8082",
            password: EndpointPassword::dev(),
        })
    }

    /// Returns the current environment.
    ///
    /// If the ROSWAAL_ENV environment variable is "dev", then the development environment is used.
    /// Otherwise, the production environment is used.
    pub async fn current() -> Result<Self> {
        if RoswaalEnvironement::current() == RoswaalEnvironement::Dev {
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

    pub fn github_pull_request_open(&self) -> &impl GithubPullRequestOpen {
        self.http_client.as_ref()
    }

    pub fn slack_messenger(&self) -> Arc<impl SlackSendMessage + Send + Sync + 'static> {
        self.http_client.clone()
    }

    pub fn sqlite(&self) -> Arc<RoswaalSqlite> {
        self.sqlite.clone()
    }

    pub fn address(&self) -> String {
        self.address.to_string()
    }

    pub fn password(&self) -> EndpointPassword {
        self.password.clone()
    }
}
