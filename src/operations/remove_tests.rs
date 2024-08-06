use std::{error::Error, fmt::Display, future};

use anyhow::Result;
use tokio::{fs::remove_dir_all, spawn};

use crate::{
    git::{
        branch_name::RoswaalOwnedGitBranchName,
        edit::EditGitRepositoryStatus,
        metadata::RoswaalGitRepositoryMetadata,
        pull_request::GithubPullRequestOpen,
        repo::{RoswaalGitRepository, RoswaalGitRepositoryClient},
    },
    tests_data::query::RoswaalTestNamesString,
    utils::{dedup::DedupIterator, sqlite::RoswaalSqlite},
    with_transaction,
};

#[derive(Debug, PartialEq, Eq)]
pub enum RemoveTestsStatus {
    Success {
        removed_test_names: Vec<String>,
        should_warn_undeleted_branch: bool,
    },
    NoTestsRemoved,
    FailedToOpenPullRequest,
    MergeConflict,
}

impl RemoveTestsStatus {
    pub async fn from_removing_tests(
        query_str: &str,
        sqlite: &RoswaalSqlite,
        git_repository: &RoswaalGitRepository<impl RoswaalGitRepositoryClient>,
        pr_open: &impl GithubPullRequestOpen,
    ) -> Result<Self> {
        let test_names = RoswaalTestNamesString::new(query_str);
        if test_names.is_empty() {
            return Ok(Self::NoTestsRemoved);
        }

        let transaction = git_repository.transaction().await;
        let branch_name = RoswaalOwnedGitBranchName::for_removing_tests();
        let metadata = transaction.metadata().clone();
        let edit_result = EditGitRepositoryStatus::from_editing_new_branch(
            &branch_name,
            transaction,
            pr_open,
            async {
                let removed_test_names = Self::remove_test_names(&test_names, &metadata).await?;
                let pr = metadata.remove_tests_pull_request(&test_names, &branch_name);
                Ok((pr, removed_test_names))
            },
        )
        .await;

        match edit_result {
            Ok(EditGitRepositoryStatus::Success {
                did_delete_branch,
                value: removed_test_names,
            }) => {
                let mut transaction = sqlite.transaction().await?;
                with_transaction!(transaction, async {
                    transaction
                        .stage_test_removals(&test_names, &branch_name)
                        .await?;
                    Ok(Self::Success {
                        removed_test_names,
                        should_warn_undeleted_branch: !did_delete_branch,
                    })
                })
            }
            Ok(EditGitRepositoryStatus::MergeConflict) => Ok(Self::MergeConflict),
            Ok(EditGitRepositoryStatus::FailedToOpenPullRequest) => {
                Ok(Self::FailedToOpenPullRequest)
            }
            Err(err) => {
                let _: NoTestsToRemoveError = err.downcast()?;
                Ok(Self::Success {
                    removed_test_names: vec![],
                    should_warn_undeleted_branch: true,
                })
            }
        }
    }

    async fn remove_test_names(
        test_names: &RoswaalTestNamesString<'_>,
        metadata: &RoswaalGitRepositoryMetadata,
    ) -> Result<Vec<String>> {
        let futures = test_names.iter().dedup().map(|n| {
            let name = n.to_string();
            let dir_path = metadata.test_dirpath(n);
            spawn(async {
                remove_dir_all(dir_path).await?;
                Ok::<String, anyhow::Error>(name)
            })
        });
        let mut removed_test_names = Vec::<String>::new();
        for future in futures {
            if let Ok(name) = future.await? {
                removed_test_names.push(name)
            }
        }
        if removed_test_names.is_empty() {
            Err(anyhow::Error::new(NoTestsToRemoveError))
        } else {
            Ok(removed_test_names)
        }
    }
}

#[derive(Debug)]
struct NoTestsToRemoveError;

impl Display for NoTestsToRemoveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NoTestsToRemoveError")
    }
}

impl Error for NoTestsToRemoveError {}

#[cfg(test)]
mod tests {
    use tokio::fs::try_exists;

    use super::*;
    use crate::{
        git::{
            metadata::RoswaalGitRepositoryMetadata,
            test_support::{
                with_clean_test_repo_access, NoopGitRepositoryClient, TestGithubPullRequestOpen,
            },
        },
        operations::{add_tests::AddTestsStatus, merge_branch::MergeBranchStatus},
        utils::sqlite::RoswaalSqlite,
    };

    #[tokio::test]
    async fn reports_no_test_removed_when_empty_query_string() {
        let status = RemoveTestsStatus::from_removing_tests(
            "",
            &RoswaalSqlite::in_memory().await.unwrap(),
            &RoswaalGitRepository::noop().await.unwrap(),
            &TestGithubPullRequestOpen::new(false),
        )
        .await
        .unwrap();
        assert_eq!(status, RemoveTestsStatus::NoTestsRemoved)
    }

    #[tokio::test]
    async fn removes_code_generated_by_adding_test() {
        with_clean_test_repo_access(async {
            let sqlite = RoswaalSqlite::in_memory().await?;
            let metadata = RoswaalGitRepositoryMetadata::for_testing();
            let repo = RoswaalGitRepository::noop().await?;
            let pr_open = TestGithubPullRequestOpen::new(false);
            add_and_merge_blob(&sqlite, &repo, &pr_open).await?;
            remove_blob(&sqlite, &repo, &pr_open).await?;
            assert!(!try_exists(metadata.relative_path("roswaal/blob")).await?);
            Ok(())
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn pr_not_opened_when_no_test_names_exist() {
        with_clean_test_repo_access(async {
            let sqlite = RoswaalSqlite::in_memory().await?;
            let repo = RoswaalGitRepository::noop().await?;
            let pr_open = TestGithubPullRequestOpen::new(false);
            _ = remove_blob(&sqlite, &repo, &pr_open).await?;
            assert!(pr_open.most_recent_pr().await.is_none());
            Ok(())
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn pr_opened_when_test_cases_removed() {
        with_clean_test_repo_access(async {
            let sqlite = RoswaalSqlite::in_memory().await?;
            let repo = RoswaalGitRepository::noop().await?;
            let pr_open = TestGithubPullRequestOpen::new(false);
            add_and_merge_blob(&sqlite, &repo, &pr_open).await?;
            _ = remove_blob(&sqlite, &repo, &pr_open).await?;
            assert!(pr_open
                .most_recent_pr()
                .await
                .unwrap()
                .title()
                .contains("Remove Tests Blob"));
            Ok(())
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn reports_tests_names_that_were_removed_in_success_status() {
        with_clean_test_repo_access(async {
            let sqlite = RoswaalSqlite::in_memory().await?;
            let repo = RoswaalGitRepository::noop().await?;
            let pr_open = TestGithubPullRequestOpen::new(false);
            add_and_merge_blob(&sqlite, &repo, &pr_open).await?;
            let test_names_str = "\
Blob
Zanza The Divine
";
            let status =
                RemoveTestsStatus::from_removing_tests(test_names_str, &sqlite, &repo, &pr_open)
                    .await?;
            let expected_status = RemoveTestsStatus::Success {
                removed_test_names: vec!["Blob".to_string()],
                should_warn_undeleted_branch: false,
            };
            assert_eq!(status, expected_status);
            assert!(pr_open
                .most_recent_pr()
                .await
                .unwrap()
                .title()
                .contains("Remove Tests Blob"));
            Ok(())
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn reports_pr_open_failed_status_when_failing_to_open_pr() {
        with_clean_test_repo_access(async {
            let sqlite = RoswaalSqlite::in_memory().await?;
            let pr_open = TestGithubPullRequestOpen::new(true);
            let repo = RoswaalGitRepository::noop().await?;
            add_and_merge_blob(&sqlite, &repo, &pr_open).await?;
            let status = remove_blob(&sqlite, &repo, &pr_open).await?;
            assert_eq!(status, RemoveTestsStatus::FailedToOpenPullRequest);
            Ok(())
        })
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn reports_merge_conflict_status_when_merge_conflict_occurs() {
        with_clean_test_repo_access(async {
            let sqlite = RoswaalSqlite::in_memory().await?;
            let pr_open = TestGithubPullRequestOpen::new(false);
            let repo = RoswaalGitRepository::noop_ensuring_merge_conflicts().await?;
            let status = remove_blob(&sqlite, &repo, &pr_open).await?;
            assert_eq!(status, RemoveTestsStatus::MergeConflict);
            Ok(())
        })
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn does_not_include_duplicate_removed_test_names() {
        with_clean_test_repo_access(async {
            let sqlite = RoswaalSqlite::in_memory().await?;
            let pr_open = TestGithubPullRequestOpen::new(false);
            let repo = RoswaalGitRepository::noop().await?;
            let tests_str = "\
```
New Test: Blob
Step 1: Do the thing
Requirement 1: Do the thing
```
```
New Test: Blob 2
Step 1: Step A
Requirement 1: Requirement A
```
";
            add_and_merge(tests_str, &sqlite, &repo, &pr_open).await?;
            let status =
                RemoveTestsStatus::from_removing_tests("Blob\nBlob", &sqlite, &repo, &pr_open)
                    .await?;
            match status {
                RemoveTestsStatus::Success {
                    removed_test_names,
                    should_warn_undeleted_branch: _,
                } => {
                    assert_eq!(removed_test_names, vec!["Blob"])
                }
                _ => panic!(),
            }
            Ok(())
        })
        .await
        .unwrap()
    }

    async fn add_and_merge_blob(
        sqlite: &RoswaalSqlite,
        repo: &RoswaalGitRepository<NoopGitRepositoryClient>,
        pr_open: &TestGithubPullRequestOpen,
    ) -> Result<()> {
        let tests_str = "\
```
New Test: Blob
Step 1: Do the thing
Requirement 1: Do the thing
```
";
        add_and_merge(tests_str, sqlite, repo, pr_open).await
    }

    async fn add_and_merge(
        tests_str: &str,
        sqlite: &RoswaalSqlite,
        repo: &RoswaalGitRepository<NoopGitRepositoryClient>,
        pr_open: &TestGithubPullRequestOpen,
    ) -> Result<()> {
        AddTestsStatus::from_adding_tests(tests_str, sqlite, pr_open, repo).await?;
        MergeBranchStatus::from_merging_branch_with_name(
            &pr_open.most_recent_head_branch_name().await.unwrap(),
            &sqlite,
        )
        .await?;
        Ok(())
    }

    async fn remove_blob(
        sqlite: &RoswaalSqlite,
        repo: &RoswaalGitRepository<NoopGitRepositoryClient>,
        pr_open: &TestGithubPullRequestOpen,
    ) -> Result<RemoveTestsStatus> {
        RemoveTestsStatus::from_removing_tests("Blob", sqlite, repo, pr_open).await
    }
}
