use std::sync::Arc;
use anyhow::Result;

#[cfg(test)]
use tokio::sync::Mutex;

use super::{branch_name::RoswaalOwnedGitBranchName, metadata::RoswaalGitRepositoryMetadata, pull_request::{GithubPullRequest, GithubPullRequestOpen}, repo::{PullBranchStatus, RoswaalGitRepositoryClient, RoswaalGitRepository, LibGit2RepositoryClient}};

#[cfg(test)]
pub struct TestGithubPullRequestOpen {
    mutex: Arc<Mutex<Option<GithubPullRequest>>>,
    should_fail: bool
}

#[cfg(test)]
impl TestGithubPullRequestOpen {
    pub fn new(should_fail: bool) -> Self {
        Self { mutex: Arc::new(Mutex::new(None)), should_fail }
    }
}

#[cfg(test)]
impl TestGithubPullRequestOpen {
    pub async fn most_recent_pr(&self) -> Option<GithubPullRequest> {
        let pr = self.mutex.lock().await;
        pr.clone()
    }
}

#[cfg(test)]
impl GithubPullRequestOpen for TestGithubPullRequestOpen {
    async fn open(&self, pull_request: &GithubPullRequest) -> Result<bool> {
        let mut pr = self.mutex.lock().await;
        *pr = Some(pull_request.clone());
        Ok(!self.should_fail)
    }
}

/// A `RoswaalGitRepositoryClient` implementation suitable for test-stubbing.
#[cfg(test)]
pub struct NoopGitRepositoryClient;

#[cfg(test)]
impl RoswaalGitRepositoryClient for NoopGitRepositoryClient {
    async fn try_new(_: &RoswaalGitRepositoryMetadata) -> Result<Self> {
        Ok(Self)
    }

    async fn hard_reset_to_head(&self) -> Result<()> {
        Ok(())
    }

    async fn switch_branch(&self, _: &str) -> Result<()> {
        Ok(())
    }

    async fn pull_branch(&self, _: &str) -> Result<PullBranchStatus> {
        Ok(PullBranchStatus::Success)
    }

    async fn commit_all(&self, _: &str) -> Result<()> {
        Ok(())
    }

    async fn checkout_new_branch(&self, _: &RoswaalOwnedGitBranchName) -> Result<()> {
        Ok(())
    }

    async fn push_changes(&self, _: &RoswaalOwnedGitBranchName) -> Result<()> {
        Ok(())
    }

    async fn clean_all_untracked(&self) -> Result<()> {
        Ok(())
    }

    async fn delete_local_branch(&self, _: &RoswaalOwnedGitBranchName) -> Result<bool> {
        Ok(true)
    }
}

impl RoswaalGitRepository<NoopGitRepositoryClient> {
    pub async fn noop() -> Result<Self> {
        Self::open(&RoswaalGitRepositoryMetadata::for_testing()).await
    }
}

#[cfg(test)]
pub async fn repo_with_test_metadata() -> Result<(
    RoswaalGitRepository::<LibGit2RepositoryClient>,
    RoswaalGitRepositoryMetadata
)> {
    let metadata = RoswaalGitRepositoryMetadata::for_testing();
    let repo = RoswaalGitRepository::<LibGit2RepositoryClient>::open(&metadata).await?;
    Ok((repo, metadata))
}
