use anyhow::Result;
use dotenv::dotenv;
use once_cell::sync::Lazy;
use std::future::Future;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;

#[cfg(test)]
use tokio::sync::Mutex;

use super::{
    branch_name::RoswaalOwnedGitBranchName,
    metadata::{self, RoswaalGitRepositoryMetadata},
    pull_request::{GithubPullRequest, GithubPullRequestOpen},
    repo::{
        LibGit2RepositoryClient, PullBranchStatus, RoswaalGitRepository, RoswaalGitRepositoryClient,
    },
};

#[cfg(test)]
pub struct TestGithubPullRequestOpen {
    mutex: Arc<Mutex<Option<GithubPullRequest>>>,
    should_fail: bool,
}

#[cfg(test)]
impl TestGithubPullRequestOpen {
    pub fn new(should_fail: bool) -> Self {
        Self {
            mutex: Arc::new(Mutex::new(None)),
            should_fail,
        }
    }
}

#[cfg(test)]
impl TestGithubPullRequestOpen {
    pub async fn most_recent_pr(&self) -> Option<GithubPullRequest> {
        let pr = self.mutex.lock().await;
        pr.clone()
    }

    pub async fn most_recent_head_branch_name(&self) -> Option<RoswaalOwnedGitBranchName> {
        self.most_recent_pr()
            .await
            .map(|pr| pr.head_branch().clone())
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
pub struct NoopGitRepositoryClient {
    metadata: RoswaalGitRepositoryMetadata,
    should_merge_conflict: bool,
}

impl NoopGitRepositoryClient {
    pub fn ensure_merge_conflict(&mut self) {
        self.should_merge_conflict = true
    }
}

#[cfg(test)]
impl RoswaalGitRepositoryClient for NoopGitRepositoryClient {
    async fn try_new(metadata: &RoswaalGitRepositoryMetadata) -> Result<Self> {
        Ok(Self {
            metadata: metadata.clone(),
            should_merge_conflict: false,
        })
    }

    fn metadata(&self) -> &RoswaalGitRepositoryMetadata {
        &self.metadata
    }

    async fn hard_reset_to_head(&self) -> Result<()> {
        Ok(())
    }

    async fn switch_branch(&self, _: &str) -> Result<()> {
        Ok(())
    }

    async fn pull_branch(&self, _: &str) -> Result<PullBranchStatus> {
        if self.should_merge_conflict {
            Ok(PullBranchStatus::MergeConflict)
        } else {
            Ok(PullBranchStatus::Success)
        }
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

    pub async fn noop_ensuring_merge_conflicts() -> Result<Self> {
        let repo = Self::open(&RoswaalGitRepositoryMetadata::for_testing()).await?;
        let mut transaction = repo.transaction().await;
        transaction.ensure_merge_conflict();
        drop(transaction);
        Ok(repo)
    }
}

#[cfg(test)]
pub async fn repo_with_test_metadata() -> Result<(
    RoswaalGitRepository<LibGit2RepositoryClient>,
    RoswaalGitRepositoryMetadata,
)> {
    let metadata = RoswaalGitRepositoryMetadata::for_testing();
    let repo = RoswaalGitRepository::<LibGit2RepositoryClient>::open(&metadata).await?;
    Ok((repo, metadata))
}

#[cfg(test)]
pub async fn write_string(path: &str, contents: &str) -> Result<()> {
    let mut file = File::create(path).await?;
    file.write(contents.as_bytes()).await?;
    file.flush().await?;
    Ok(drop(file))
}

#[cfg(test)]
pub async fn read_string(path: &str) -> Result<String> {
    let mut file = File::open(path).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    Ok(contents)
}

#[cfg(test)]
static TEST_REPO_LOCK: Lazy<Arc<Mutex<()>>> = Lazy::new(|| Arc::new(Mutex::new(())));

/// Cleans and serializes access to the test repo for the duration of the future.
#[cfg(test)]
pub async fn with_clean_test_repo_access(work: impl Future<Output = Result<()>>) -> Result<()> {
    let guard = TEST_REPO_LOCK.lock().await;
    dotenv()?;
    Command::new("./reset_test_repo.sh").spawn()?.wait().await?;
    let result = work.await;
    drop(guard);
    result
}
