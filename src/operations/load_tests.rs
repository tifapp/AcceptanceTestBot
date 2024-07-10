use anyhow::Result;

use crate::{tests_data::storage::RoswaalStoredTest, utils::sqlite::RoswaalSqlite, with_transaction};

#[derive(Debug, PartialEq, Eq)]
pub enum LoadTestsStatus {
    Success(Vec<RoswaalStoredTest>),
    NoTests
}

impl LoadTestsStatus {
    pub async fn for_loading_tests(sqlite: &RoswaalSqlite) -> Result<Self> {
        let mut transaction = sqlite.transaction().await?;
        with_transaction!(transaction, async {
            let tests = transaction.tests_in_alphabetical_order().await?;
            if tests.is_empty() {
                Ok(Self::NoTests)
            } else {
                Ok(Self::Success(tests))
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{git::{repo::RoswaalGitRepository, test_support::{with_clean_test_repo_access, TestGithubPullRequestOpen}}, operations::add_tests::AddTestsStatus, utils::sqlite::RoswaalSqlite};

    #[tokio::test]
    async fn reports_no_tests_when_no_tests_saved() {
        let sqlite = RoswaalSqlite::in_memory().await.unwrap();
        let status = LoadTestsStatus::for_loading_tests(&sqlite).await.unwrap();
        assert_eq!(status, LoadTestsStatus::NoTests)
    }

    #[tokio::test]
    async fn reports_success_with_saved_tests() {
        with_clean_test_repo_access(async {
            let sqlite = RoswaalSqlite::in_memory().await?;
            let tests_str = "\
```
New Test: ABC
Step 1: Do the thing
Requirement 1: Do the thing
```
```
New Test: ABC 123
Step 1: Do the thing
Requirement 1: Do the thing
```
";
            _ = AddTestsStatus::from_adding_tests(
                tests_str,
                &sqlite,
                &TestGithubPullRequestOpen::new(false),
                &RoswaalGitRepository::noop().await?
            ).await?;
            let status = LoadTestsStatus::for_loading_tests(&sqlite).await?;
            match status {
                LoadTestsStatus::Success(tests) => {
                    assert_eq!(tests[0].name(), "ABC");
                    assert_eq!(tests[1].name(), "ABC 123")
                },
                _ => panic!()
            }
            Ok(())
        })
        .await.unwrap();
    }
}
