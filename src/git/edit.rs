use std::future::Future;

use anyhow::Result;

use super::{branch_name::RoswaalOwnedGitBranchName, pull_request::{GithubPullRequest, GithubPullRequestOpen}, repo::{RoswaalGitRepository, RoswaalGitRepositoryClient}};

#[derive(Debug, PartialEq, Eq)]
pub enum EditGitRepositoryStatus {
    Success { did_delete_branch: bool },
    FailedToOpenPullRequest,
    MergeConflict
}

impl EditGitRepositoryStatus {
    pub async fn from_editing_new_branch(
        new_branch_name: &RoswaalOwnedGitBranchName,
        base_branch_name: &str,
        repo: &RoswaalGitRepository<impl RoswaalGitRepositoryClient>,
        pr_open: &impl GithubPullRequestOpen,
        edit: impl Future<Output = Result<GithubPullRequest>>
    ) -> Result<Self> {
        let transaction = repo.transaction().await;
        transaction.checkout_new_branch(new_branch_name).await?;
        let pull_request = edit.await?;
        transaction.commit_all(pull_request.title()).await?;
        transaction.push_changes(new_branch_name).await?;
        transaction.switch_branch(base_branch_name).await?;
        let did_delete_branch = transaction.delete_local_branch(new_branch_name).await?;
        drop(transaction);
        pr_open.open(&pull_request).await?;
        Ok(Self::Success { did_delete_branch })
    }
}

#[cfg(test)]
mod tests {
    use tokio::fs::{try_exists, File};

    use super::*;
    use crate::{git::{repo::PullBranchStatus, test_support::{repo_with_test_metadata, TestGithubPullRequestOpen}}, utils::test_support::with_clean_test_repo_access};

    #[tokio::test]
    async fn test_basic_flow_returns_successfully_with_proper_pr_opened_and_correct_repo_state() {
        with_clean_test_repo_access(async {
            let (repo, metadata) = repo_with_test_metadata().await?;
            let pr_open = TestGithubPullRequestOpen::new(false);
            let new_branch_name = RoswaalOwnedGitBranchName::new("test-edit");
            let expected_pr = GithubPullRequest::for_tif_react_frontend("Hello", "World", &new_branch_name);
            let ret_pr = expected_pr.clone();
            let file_path = metadata.relative_path("test-thing.txt");
            let status = EditGitRepositoryStatus::from_editing_new_branch(
                &new_branch_name,
                metadata.base_branch_name(),
                &repo,
                &pr_open,
                async {
                    File::create(&file_path).await?;
                    Ok(ret_pr)
                }
            ).await?;

            assert_eq!(status, EditGitRepositoryStatus::Success { did_delete_branch: true });
            assert_eq!(pr_open.most_recent_pr().await, Some(expected_pr));

            let transaction = repo.transaction().await;

            let switch_result = transaction.switch_branch(&new_branch_name.to_string()).await;
            assert!(switch_result.is_err(), "The new local branch should've been deleted, so we cannot switch to it.");

            assert!(!try_exists(&file_path).await?);
            let status = transaction.pull_branch(&new_branch_name.to_string()).await?;
            assert_eq!(status, PullBranchStatus::Success);
            assert!(try_exists(&file_path).await?);

            Ok(())
        })
        .await.unwrap()
    }
}
