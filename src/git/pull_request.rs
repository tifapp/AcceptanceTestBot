use std::env;

use anyhow::Result;
use reqwest::{header::CONTENT_TYPE, Client};
use serde::Serialize;

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct GithubPullRequest {
    title: String,
    body: String,
    #[serde(skip)]
    owner: String,
    #[serde(skip)]
    repo: String,
    head: String,
    base: String
}

impl GithubPullRequest {
    /// Creates a PR for the main frontend repo.
    pub fn for_tif_react_frontend(title: String, body: String, head_branch: String) -> Self {
        Self {
            body,
            title: format!("Roswaal: {}", title),
            owner: "tifapp".to_string(),
            repo: "FitnessProject".to_string(),
            base: "development".to_string(),
            head: format!("roswaal:{}", head_branch)
        }
    }
}

pub trait GithubPullRequestOpen {
    /// Opens a PR on github, and returns true if it was created successfully.
    async fn open(&self, pull_request: &GithubPullRequest) -> Result<bool>;
}

impl GithubPullRequestOpen for Client {
    async fn open(&self, pull_request: &GithubPullRequest) -> Result<bool> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls",
            pull_request.owner,
            pull_request.repo
        );
        let response = self.post(url)
            .bearer_auth(env::var("GITHUB_API_KEY").unwrap())
            .header(CONTENT_TYPE, "application/json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .json(&pull_request)
            .send()
            .await?;
        Ok(response.status() == 201)
    }
}
