use std::iter::zip;

use anyhow::Result;
use tokio::spawn;

use crate::{generation::interface::RoswaalTypescriptGenerate, git::{branch_name::{self, RoswaalOwnedGitBranchName}, edit::EditGitRepositoryStatus, metadata::{self, RoswaalGitRepositoryMetadata}, pull_request::GithubPullRequestOpen, repo::{RoswaalGitRepository, RoswaalGitRepositoryClient}}, language::{ast::{extract_tests_syntax, RoswaalTestSyntax}, compiler::{RoswaalCompilationError, RoswaalCompile, RoswaalCompileContext}, test::RoswaalTest}, location::{name::RoswaalLocationName, storage::LoadLocationsFilter}, utils::sqlite::RoswaalSqlite, with_transaction};

#[derive(Debug, PartialEq, Eq)]
pub enum AddTestsStatus {
    Success { results: Vec<Result<RoswaalTest, Vec<RoswaalCompilationError>>>, did_delete_branch: bool },
    NoTestsFound,
    MergeConflict,
    FailedToOpenPullRequest
}

impl AddTestsStatus {
    pub async fn from_adding_tests(
        tests_str: &str,
        sqlite: &RoswaalSqlite,
        pr_open: &impl GithubPullRequestOpen,
        git_repository: &RoswaalGitRepository<impl RoswaalGitRepositoryClient>
    ) -> Result<Self> {
        let tests_syntax = extract_tests_syntax(tests_str);
        if tests_syntax.is_empty() { return Ok(AddTestsStatus::NoTestsFound) }
        let mut transaction = sqlite.transaction().await?;
        let (location_names, git_transaction) = with_transaction!(transaction, async {
            let location_names = transaction.locations_in_alphabetical_order(
                LoadLocationsFilter::MergedOnly
            )
            .await?
            .iter()
            .map(|l| l.location().name().clone())
            .collect::<Vec<RoswaalLocationName>>();
            Ok((location_names, git_repository.transaction().await))
        })?;
        let branch_name = RoswaalOwnedGitBranchName::for_adding_tests();
        let metadata = git_transaction.metadata().clone();
        let cloned_tests_syntax = tests_syntax.clone();
        let results = cloned_tests_syntax
            .iter()
            .map(|syntax| {
                let compile_context = RoswaalCompileContext::new(&location_names);
                RoswaalTest::compile_syntax(syntax, compile_context)
            })
            .collect::<Vec<Result<RoswaalTest, Vec<RoswaalCompilationError>>>>();
        if results.iter().filter(|r| r.is_ok()).count() == 0 {
            return Ok(Self::Success { results, did_delete_branch: false })
        }
        let edit_status = EditGitRepositoryStatus::from_editing_new_branch(
            &branch_name,
            git_transaction,
            pr_open,
            async {
                let futures = results.iter().filter_map(|r| {
                    if let Ok(test) = r {
                        let test = test.clone();
                        let metadata = metadata.clone();
                        Some(
                            spawn(async move {
                                test.typescript().save_in_dir(&metadata.test_dirpath(&test)).await
                            })
                        )
                    } else {
                        None
                    }
                });
                for future in futures {
                    future.await??;
                }
                let test_names_with_syntax = zip(results.iter(), tests_syntax.iter())
                    .filter_map(|(r, syntax)| {
                        if let Ok(test) = r {
                            Some((test.name(), syntax))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<(&str, &RoswaalTestSyntax<'_>)>>();
                Ok(metadata.add_tests_pull_request(&test_names_with_syntax, &branch_name))
            }
        ).await?;
        match edit_status {
            EditGitRepositoryStatus::Success { did_delete_branch } => {
                transaction = sqlite.transaction().await?;
                with_transaction!(transaction, async {
                    let tests = results.iter()
                        .filter_map(|r| r.clone().ok())
                        .collect::<Vec<RoswaalTest>>();
                    transaction.save_tests(&tests, &branch_name).await?;
                    Ok(Self::Success { results, did_delete_branch })
                })
            },
            EditGitRepositoryStatus::FailedToOpenPullRequest => {
                Ok(Self::FailedToOpenPullRequest)
            },
            EditGitRepositoryStatus::MergeConflict => {
                Ok(Self::MergeConflict)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{git::{branch_name, metadata::RoswaalGitRepositoryMetadata, repo::RoswaalGitRepository, test_support::{read_string, with_clean_test_repo_access, TestGithubPullRequestOpen}}, language::{compiler::RoswaalCompilationErrorCode, test::RoswaalTestCommand}, operations::{add_locations::AddLocationsStatus, merge_branch::MergeBranchStatus}, utils::sqlite::RoswaalSqlite};

    #[tokio::test]
    async fn reports_results_of_compiling_multiple_tests() {
        with_clean_test_repo_access(async {
            let tests_str = "\
```
New Test: Basic Leave Event through Exploration as Attendee
Abstract: Justin is looking for an event. Once he finds one, he realizes that her has other plans, and decides to leave.

Step 1: Justin is signed in
Step 2: Justin wants to find the nearest event
Step 3: After finding an event, Justin wants to join it
Step 4: After some pondering, Justin decides that he is not interested in the event and wants to leave
Step 5: Justin has now left the event

Requirement 1: Ensure Justin has signed into his account
Requirement 2: Search for the nearest events, and go to the details for the nearest one
Requirement 3: Have Justin join the event
Requirement 4: Have Justin leave the event
Requirement 5: Ensure that Justin has left the event successfully
```
```
This is like an invalid test or something...
```
";
            let status = AddTestsStatus::from_adding_tests(
                tests_str,
                &RoswaalSqlite::in_memory().await?,
                &TestGithubPullRequestOpen::new(false),
                 &RoswaalGitRepository::noop().await?
            ).await?;
            let expected_results = vec![
                Ok(
                    RoswaalTest::new(
                        "Basic Leave Event through Exploration as Attendee".to_string(),
                        Some("Justin is looking for an event. Once he finds one, he realizes that her has other plans, and decides to leave.".to_string()),
                        vec![
                            RoswaalTestCommand::Step {
                                name: "Justin is signed in".to_string(),
                                requirement: "Ensure Justin has signed into his account".to_string()
                            },
                            RoswaalTestCommand::Step {
                                name: "Justin wants to find the nearest event".to_string(),
                                requirement: "Search for the nearest events, and go to the details for the nearest one".to_string()
                            },
                            RoswaalTestCommand::Step {
                                name: "After finding an event, Justin wants to join it".to_string(),
                                requirement: "Have Justin join the event".to_string()
                            },
                            RoswaalTestCommand::Step {
                                name: "After some pondering, Justin decides that he is not interested in the event and wants to leave".to_string(),
                                requirement: "Have Justin leave the event".to_string()
                            },
                            RoswaalTestCommand::Step {
                                name: "Justin has now left the event".to_string(),
                                requirement: "Ensure that Justin has left the event successfully".to_string()
                            }
                        ]
                    )
                ),
                Err(
                    vec![
                        RoswaalCompilationError::new(
                            1,
                            RoswaalCompilationErrorCode::InvalidCommandName(
                                "This is like an invalid test or something...".to_string()
                            )
                        ),
                        RoswaalCompilationError::new(1, RoswaalCompilationErrorCode::NoTestName)
                    ]
                )
            ];
            assert_eq!(status, AddTestsStatus::Success { results: expected_results, did_delete_branch: true });
            Ok(())
        })
        .await.unwrap();
    }

    #[tokio::test]
    async fn only_uses_merged_location_names_when_compiling_tests() {
        with_clean_test_repo_access(async {
            let tests_str = "\
```
New Test: ABC

Step 1: Do the thing
Requirement 1: Do the thing
Set Location: Test 2
```
```
New Test: ABC 123

Step 1: Do the thing
Requirement 1: Do the thing
Set Location: Test
```
";
            let sqlite = RoswaalSqlite::in_memory().await?;
            let repo = RoswaalGitRepository::noop().await?;
            let pr_open = TestGithubPullRequestOpen::new(false);
            _ = AddLocationsStatus::from_adding_locations(
                "Test, 5.0, 5.0",
                &repo,
                &sqlite,
                &pr_open
            ).await?;
            _ = AddLocationsStatus::from_adding_locations(
                "Test 2, 5.0, 5.0",
                &repo,
                &sqlite,
                &pr_open
            ).await?;
            let branch_name = pr_open.most_recent_head_branch_name().await.unwrap();
            _ = MergeBranchStatus::from_merging_branch_with_name(&branch_name, &sqlite).await?;
            let status = AddTestsStatus::from_adding_tests(
                tests_str,
                &sqlite,
                &TestGithubPullRequestOpen::new(false),
                &RoswaalGitRepository::noop().await?
            ).await?;
            match status {
                AddTestsStatus::Success { results, did_delete_branch: _ } => {
                    assert!(results[0].is_ok());
                    assert!(results[1].is_err());
                },
                _ => panic!()
            }
            Ok(())
        })
        .await.unwrap()
    }

    #[tokio::test]
    async fn generates_test_case_and_test_action_code() {
        with_clean_test_repo_access(async {
            let tests_str = "\
```
New Test: ABC 123
Step 1: Do the thing
Requirement 1: Do the thing
```
";
            let metadata = RoswaalGitRepositoryMetadata::for_testing();
            let sqlite = RoswaalSqlite::in_memory().await?;
            _ = AddTestsStatus::from_adding_tests(
                tests_str,
                &sqlite,
                &TestGithubPullRequestOpen::new(false),
                 &RoswaalGitRepository::noop().await?
            ).await?;
            let test_case_code = read_string(
                &metadata.relative_path("roswaal/abc-123/TestCase.test.ts")
            ).await?;
            let test_action_code = read_string(
                &metadata.relative_path("roswaal/abc-123/TestActions.ts")
            ).await?;
            let expected_test_case_code = "\
// Generated by Roswaal, do not touch.

import * as TestActions from \"./TestActions\"
import { launchApp } from \"../Launch\"
import { RoswaalTestCase } from \"../TestCase\"
import { roswaalClient } from \"../Client\"

test(\"ABC 123\", async () => {
  const testCase = new RoswaalTestCase(\"ABC 123\", TestActions.beforeLaunch)
  // Do the thing
  testCase.appendAction(TestActions.doTheThing)
  await roswaalClient.run(testCase)
})
";
            let expected_test_actions_code = "\
import { TestAppLaunchConfig } from \"../Launch\"

export const beforeLaunch = async (): Promise<TestAppLaunchConfig> => {
  // Perform any setup work in here, (setting location, reseting device
  // permissions, etc.)
  return {}
}

export const doTheThing = async () => {
  // Do the thing
  throw new Error(\"TODO\")
}
";
            assert_eq!(test_case_code, expected_test_case_code);
            assert_eq!(test_action_code, expected_test_actions_code);
            Ok(())
        })
        .await.unwrap()
    }

    #[tokio::test]
    async fn opens_pr_for_adding_new_tests() {
        with_clean_test_repo_access(async {
            let tests_str = "\
```
New Test: ABC 123
Step 1: Do the thing
Requirement 1: Do the thing
```
```
New Test: I am the strong
Step 1: Do the thing
Requirement 1: Do the thing
```
";
            let sqlite = RoswaalSqlite::in_memory().await?;
            let pr_open = TestGithubPullRequestOpen::new(false);
            _ = AddTestsStatus::from_adding_tests(
                tests_str,
                &sqlite,
                &pr_open,
                 &RoswaalGitRepository::noop().await?
            ).await?;
            let pr = pr_open.most_recent_pr().await.unwrap();
            assert!(pr.title().contains("Add Tests \"ABC 123\", \"I am the strong\""));
            Ok(())
        })
        .await.unwrap()
    }

    #[tokio::test]
    async fn reports_pr_open_failed_status_when_failing_to_open_pr() {
        with_clean_test_repo_access(async {
            let tests_str = "\
```
New Test: ABC 123
Step 1: Do the thing
Requirement 1: Do the thing
```
";
            let sqlite = RoswaalSqlite::in_memory().await?;
            let pr_open = TestGithubPullRequestOpen::new(true);
            let status = AddTestsStatus::from_adding_tests(
                tests_str,
                &sqlite,
                &pr_open,
                &RoswaalGitRepository::noop().await?
            ).await?;
            assert_eq!(status, AddTestsStatus::FailedToOpenPullRequest);
            Ok(())
        })
        .await.unwrap()
    }

    #[tokio::test]
    async fn reports_merge_conflict_status_when_merge_conflict_occurs() {
        with_clean_test_repo_access(async {
            let tests_str = "\
```
New Test: ABC 123
Step 1: Do the thing
Requirement 1: Do the thing
```
";
            let sqlite = RoswaalSqlite::in_memory().await?;
            let pr_open = TestGithubPullRequestOpen::new(true);
            let status = AddTestsStatus::from_adding_tests(
                tests_str,
                &sqlite,
                &pr_open,
                &RoswaalGitRepository::noop_ensuring_merge_conflicts().await?
            ).await?;
            assert_eq!(status, AddTestsStatus::MergeConflict);
            Ok(())
        })
        .await.unwrap()
    }

    #[tokio::test]
    async fn reports_no_tests_found_status_when_empty_tests_string() {
        let tests_str = "";
        let sqlite = RoswaalSqlite::in_memory().await.unwrap();
        let pr_open = TestGithubPullRequestOpen::new(true);
        let status = AddTestsStatus::from_adding_tests(
            tests_str,
            &sqlite,
            &pr_open,
            &RoswaalGitRepository::noop_ensuring_merge_conflicts().await.unwrap()
        ).await.unwrap();
        assert_eq!(status, AddTestsStatus::NoTestsFound);
    }

    #[tokio::test]
    async fn does_not_open_pr_when_no_compiling_tests() {
        let tests_str = "\
```
dlkjldjlkdjlkjdlkdj
```
```
lkjdlkjlkjalkjslkdjdflkj
```
";
        let sqlite = RoswaalSqlite::in_memory().await.unwrap();
        let pr_open = TestGithubPullRequestOpen::new(true);
        _ = AddTestsStatus::from_adding_tests(
            tests_str,
            &sqlite,
            &pr_open,
            &RoswaalGitRepository::noop().await.unwrap()
        ).await.unwrap();
        let pr = pr_open.most_recent_pr().await;
        assert_eq!(pr, None)
    }
}
