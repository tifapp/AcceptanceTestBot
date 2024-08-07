use std::future::Future;

use anyhow::Result;

use super::{
    branch_name::RoswaalOwnedGitBranchName,
    pull_request::{GithubPullRequest, GithubPullRequestOpen},
    repo::{PullBranchStatus, RoswaalGitRepositoryClient, RoswaalGitRepositoryTransaction},
};

/// A status type for creating a new branch, pushing changes, opening a pull request, and
/// deleting the newly created branch.
#[derive(Debug, PartialEq, Eq)]
pub enum EditGitRepositoryStatus<T> {
    Success { did_delete_branch: bool, value: T },
    FailedToOpenPullRequest,
    MergeConflict,
}

impl<T> EditGitRepositoryStatus<T> {
    /// Performs the edit future within a git repository transaction, and opens a PR detailing
    /// those changes.
    ///
    /// Any uncomitted changes are reset and cleaned up, and the latest changes from the base
    /// branch are pulled before the edit future is ran. The new branch is deleted on the local
    /// repository after the edit is completed.
    pub async fn from_editing_new_branch(
        new_branch_name: &RoswaalOwnedGitBranchName,
        transaction: RoswaalGitRepositoryTransaction<'_, impl RoswaalGitRepositoryClient>,
        pr_open: &impl GithubPullRequestOpen,
        edit: impl Future<Output = Result<(GithubPullRequest, T)>>,
    ) -> Result<Self> {
        let base_branch_name = transaction.metadata().base_branch_name();
        transaction.hard_reset_to_head().await?;
        transaction.clean_all_untracked().await?;
        transaction.switch_branch(base_branch_name).await?;
        let pull_status = transaction.pull_branch(base_branch_name).await?;
        if pull_status == PullBranchStatus::MergeConflict {
            return Ok(Self::MergeConflict);
        }
        transaction.checkout_new_branch(new_branch_name).await?;
        match edit.await {
            Ok((pull_request, value)) => {
                transaction.commit_all(pull_request.title()).await?;
                transaction.push_changes(new_branch_name).await?;
                transaction.switch_branch(base_branch_name).await?;
                let did_delete_branch = transaction.delete_local_branch(new_branch_name).await?;
                drop(transaction);
                let did_open = pr_open.open(&pull_request).await?;
                if !did_open {
                    Ok(Self::FailedToOpenPullRequest)
                } else {
                    Ok(Self::Success {
                        did_delete_branch,
                        value,
                    })
                }
            }
            Err(err) => {
                transaction.hard_reset_to_head().await?;
                transaction.clean_all_untracked().await?;
                transaction.switch_branch(base_branch_name).await?;
                transaction.delete_local_branch(new_branch_name).await?;
                Err(err)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio::fs::{try_exists, File};

    use super::*;
    use crate::{
        git::{
            metadata::{RoswaalGitRepositoryMetadata, TEST_REPO_BASE_BRANCH_NAME},
            repo::{LibGit2RepositoryClient, PullBranchStatus, RoswaalGitRepository},
            test_support::{
                repo_with_test_metadata, with_clean_test_repo_access, write_string,
                TestGithubPullRequestOpen,
            },
        },
        utils::test_error::TestError,
    };

    #[tokio::test]
    async fn test_basic_flow_returns_successfully_with_proper_pr_opened_and_correct_repo_state() {
        with_clean_test_repo_access(async {
            let (repo, metadata) = repo_with_test_metadata().await?;
            let pr_open = TestGithubPullRequestOpen::new(false);
            let new_branch_name = RoswaalOwnedGitBranchName::new("test-edit");
            let expected_pr = GithubPullRequest::test(&new_branch_name);
            let ret_pr = expected_pr.clone();
            let file_path = metadata.relative_path("test-thing.txt");
            let status = EditGitRepositoryStatus::from_editing_new_branch(
                &new_branch_name,
                repo.transaction().await,
                &pr_open,
                async {
                    File::create(&file_path).await?;
                    Ok((ret_pr, ()))
                },
            )
            .await?;
            assert_successful_single_file_created_edit(
                &status,
                &new_branch_name,
                &expected_pr,
                &file_path,
                &repo,
                &pr_open,
            )
            .await
        })
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn test_returns_pr_open_failed_when_pr_fails_to_open() {
        with_clean_test_repo_access(async {
            let new_branch_name = RoswaalOwnedGitBranchName::new("test-edit");
            let status = EditGitRepositoryStatus::from_editing_new_branch(
                &new_branch_name,
                RoswaalGitRepository::noop().await?.transaction().await,
                &TestGithubPullRequestOpen::new(true),
                async { Ok((GithubPullRequest::test(&new_branch_name), ())) },
            )
            .await?;
            assert_eq!(status, EditGitRepositoryStatus::FailedToOpenPullRequest);
            Ok(())
        })
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn test_recovers_from_previous_failed_edit() {
        with_clean_test_repo_access(async {
            let (repo, metadata) = repo_with_test_metadata().await?;
            let pr_open = TestGithubPullRequestOpen::new(false);
            let mut branch_name = RoswaalOwnedGitBranchName::new("test-edit-recovery-failure");
            let failure_file_path = metadata.relative_path("test-failure.txt");
            let edit_result = EditGitRepositoryStatus::from_editing_new_branch(
                &branch_name,
                repo.transaction().await,
                &pr_open,
                async {
                    File::create(&failure_file_path).await?;
                    Err::<(GithubPullRequest, ()), anyhow::Error>(anyhow::Error::new(TestError))
                },
            )
            .await;
            assert!(edit_result.is_err());
            assert!(!try_exists(failure_file_path).await?);

            branch_name = RoswaalOwnedGitBranchName::new("test-edit-recovery-success");
            let expected_pr =
                GithubPullRequest::for_tif_react_frontend("Test", "Test", &branch_name);
            let edit_pr = expected_pr.clone();
            let success_file_path = metadata.relative_path("test-success.txt");
            let status = EditGitRepositoryStatus::from_editing_new_branch(
                &branch_name,
                repo.transaction().await,
                &pr_open,
                async {
                    File::create(&success_file_path).await?;
                    Ok((edit_pr, ()))
                },
            )
            .await?;
            assert_successful_single_file_created_edit(
                &status,
                &branch_name,
                &expected_pr,
                &success_file_path,
                &repo,
                &pr_open,
            )
            .await
        })
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn deletes_branch_of_failed_edit() {
        with_clean_test_repo_access(async {
            let (repo, _) = repo_with_test_metadata().await?;
            let pr_open = TestGithubPullRequestOpen::new(false);
            let branch_name = RoswaalOwnedGitBranchName::new("test-edit-delete-branch-on-failure");
            _ = EditGitRepositoryStatus::from_editing_new_branch(
                &branch_name,
                repo.transaction().await,
                &pr_open,
                async {
                    Err::<(GithubPullRequest, ()), anyhow::Error>(anyhow::Error::new(TestError))
                },
            )
            .await;

            let transaction = repo.transaction().await;
            let result = transaction.switch_branch(&branch_name.to_string()).await;
            assert!(result.is_err());
            Ok(())
        })
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn test_resets_cleans_and_pulls_latest_changes_from_base_branch_before_edit() {
        with_clean_test_repo_access(async {
            let base_branch_name = RoswaalOwnedGitBranchName::new("test-edit-pull-latest-base");
            let metadata = RoswaalGitRepositoryMetadata::for_testing_with_custom_base_branch(
                &base_branch_name.to_string(),
            );
            let repo = RoswaalGitRepository::<LibGit2RepositoryClient>::open(&metadata).await?;
            let pr_open = TestGithubPullRequestOpen::new(false);
            let file_path_1 = metadata.relative_path("test1.txt");

            let transaction = repo.transaction().await;
            transaction.checkout_new_branch(&base_branch_name).await?;
            File::create(&file_path_1).await?;
            transaction.commit_all("test").await?;
            transaction.push_changes(&base_branch_name).await?;
            transaction
                .switch_branch(TEST_REPO_BASE_BRANCH_NAME)
                .await?;
            drop(transaction);

            let transaction = repo.transaction().await;

            let file_path_2 = metadata.relative_path("test2.txt");
            File::create(&file_path_2).await?;

            let branch_name_2 = RoswaalOwnedGitBranchName::new("test-edit-pull-latest-2");
            EditGitRepositoryStatus::from_editing_new_branch(
                &branch_name_2,
                transaction,
                &pr_open,
                async {
                    assert!(try_exists(&file_path_1).await?);
                    assert!(!try_exists(&file_path_2).await?);
                    Ok((GithubPullRequest::test(&branch_name_2), ()))
                },
            )
            .await?;
            Ok(())
        })
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn test_detects_merge_conflict_when_pulling_from_base_branch() {
        with_clean_test_repo_access(async {
            let base_branch_name =
                RoswaalOwnedGitBranchName::new("test-edit-pull-merge-conflict-base");
            let metadata = RoswaalGitRepositoryMetadata::for_testing_with_custom_base_branch(
                &base_branch_name.to_string(),
            );
            let repo = RoswaalGitRepository::<LibGit2RepositoryClient>::open(&metadata).await?;
            let pr_open = TestGithubPullRequestOpen::new(false);

            let transaction = repo.transaction().await;
            transaction.checkout_new_branch(&base_branch_name).await?;
            write_string(&metadata.relative_path("test.txt"), "Hello world").await?;
            transaction.commit_all("hello").await?;
            transaction.push_changes(&base_branch_name).await?;
            transaction
                .switch_branch(TEST_REPO_BASE_BRANCH_NAME)
                .await?;
            transaction.delete_local_branch(&base_branch_name).await?;
            drop(transaction);

            // NB: The edit would've deleted base_branch_name, so we'll need to add it back so we can
            // switch to it when running the edit operation again.
            let transaction = repo.transaction().await;
            transaction.checkout_new_branch(&base_branch_name).await?;
            write_string(&metadata.relative_path("test.txt"), "Goodbye world").await?;
            transaction.commit_all("hello").await?;

            let new_branch_name = RoswaalOwnedGitBranchName::new("test-edit-pull-merge-conflict");
            let status = EditGitRepositoryStatus::from_editing_new_branch(
                &new_branch_name,
                transaction,
                &pr_open,
                async { Ok((GithubPullRequest::test(&new_branch_name), ())) },
            )
            .await?;
            assert_eq!(status, EditGitRepositoryStatus::MergeConflict);
            Ok(())
        })
        .await
        .unwrap()
    }

    async fn assert_successful_single_file_created_edit(
        status: &EditGitRepositoryStatus<()>,
        branch_name: &RoswaalOwnedGitBranchName,
        expected_pr: &GithubPullRequest,
        file_path: &str,
        repo: &RoswaalGitRepository<LibGit2RepositoryClient>,
        pr_open: &TestGithubPullRequestOpen,
    ) -> Result<()> {
        assert_eq!(
            status,
            &EditGitRepositoryStatus::Success {
                did_delete_branch: true,
                value: ()
            }
        );
        assert_eq!(pr_open.most_recent_pr().await, Some(expected_pr.clone()));

        let transaction = repo.transaction().await;

        let switch_result = transaction.switch_branch(&branch_name.to_string()).await;
        assert!(
            switch_result.is_err(),
            "The new local branch should've been deleted, so we cannot switch to it."
        );

        assert!(!try_exists(&file_path).await?);
        let status = transaction.pull_branch(&branch_name.to_string()).await?;
        assert_eq!(status, PullBranchStatus::Success);
        assert!(try_exists(&file_path).await?);
        Ok(())
    }

    impl GithubPullRequest {
        fn test(head_branch: &RoswaalOwnedGitBranchName) -> Self {
            Self::for_tif_react_frontend("Test", "Test", head_branch)
        }
    }
}
