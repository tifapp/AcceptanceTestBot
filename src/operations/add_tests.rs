use anyhow::Result;

use crate::{language::{ast::extract_tests_syntax, compiler::{RoswaalCompilationError, RoswaalCompile, RoswaalCompileContext}, test::RoswaalTest}, location::{name::RoswaalLocationName, storage::LoadLocationsFilter}, utils::sqlite::RoswaalSqlite, with_transaction};

#[derive(Debug, PartialEq, Eq)]
pub enum AddTestsStatus {
    Success(Vec<Result<RoswaalTest, Vec<RoswaalCompilationError>>>)
}

impl AddTestsStatus {
    pub async fn from_adding_tests(
        tests_str: &str,
        sqlite: &RoswaalSqlite
    ) -> Result<Self> {
        let mut transaction = sqlite.transaction().await?;
        with_transaction!(transaction, async {
            let location_names = transaction.locations_in_alphabetical_order(
                LoadLocationsFilter::MergedOnly
            )
            .await?
            .iter()
            .map(|l| l.location().name().clone())
            .collect::<Vec<RoswaalLocationName>>();
            let results = extract_tests_syntax(tests_str)
                .iter()
                .map(|syntax| {
                    let compile_context = RoswaalCompileContext::new(&location_names);
                    RoswaalTest::compile_syntax(syntax, compile_context)
                })
                .collect::<Vec<Result<RoswaalTest, Vec<RoswaalCompilationError>>>>();
            Ok(Self::Success(results))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{git::{branch_name, metadata::RoswaalGitRepositoryMetadata, repo::RoswaalGitRepository, test_support::{with_clean_test_repo_access, TestGithubPullRequestOpen}}, language::{compiler::RoswaalCompilationErrorCode, test::RoswaalTestCommand}, operations::{add_locations::AddLocationsStatus, merge_branch::MergeBranchStatus}, utils::sqlite::RoswaalSqlite};

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
                &RoswaalSqlite::in_memory().await?
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
            assert_eq!(status, AddTestsStatus::Success(expected_results));
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
            let status = AddTestsStatus::from_adding_tests(tests_str, &sqlite).await?;
            match status {
                AddTestsStatus::Success(results) => {
                    assert!(results[0].is_ok());
                    assert!(results[1].is_err());
                },
                _ => panic!()
            }
            Ok(())
        })
        .await.unwrap()
    }
}
