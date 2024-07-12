use anyhow::Result;

use crate::{tests_data::{query::RoswaalSearchTestsQuery, storage::RoswaalStoredTest}, utils::sqlite::RoswaalSqlite, with_transaction};

#[derive(Debug, PartialEq, Eq)]
pub enum SearchTestsStatus {
    Success(Vec<RoswaalStoredTest>),
    NoTests
}

impl SearchTestsStatus {
    pub async fn from_searching_tests(query_str: &str, sqlite: &RoswaalSqlite) -> Result<Self> {
        let mut transaction = sqlite.transaction().await?;
        with_transaction!(transaction, async {
            let query = RoswaalSearchTestsQuery::new(query_str);
            let tests = transaction.tests_in_alphabetical_order(&query).await?;
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
        let status = SearchTestsStatus::from_searching_tests("", &sqlite).await.unwrap();
        assert_eq!(status, SearchTestsStatus::NoTests)
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
            let status = SearchTestsStatus::from_searching_tests("", &sqlite).await?;
            match status {
                SearchTestsStatus::Success(tests) => {
                    assert_eq!(tests[0].name(), "ABC");
                    assert_eq!(tests[1].name(), "ABC 123")
                },
                _ => panic!()
            }
            Ok(())
        })
        .await.unwrap();
    }

    #[tokio::test]
    async fn reports_success_with_specific_test_names_in_query_str() {
        with_clean_test_repo_access(async {
            let sqlite = RoswaalSqlite::in_memory().await.unwrap();
            let tests_str = "\
```
New Test: Bob
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
            let query_str = "bob";
            let status = SearchTestsStatus::from_searching_tests(query_str, &sqlite).await.unwrap();
            match status {
                SearchTestsStatus::Success(tests) => {
                    assert_eq!(tests.iter().map(|t| t.name()).collect::<Vec<&str>>(), vec!["Bob"]);
                },
                _ => panic!()
            }
            Ok(())
        })
        .await.unwrap();
    }
}
