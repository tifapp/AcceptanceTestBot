use std::{fmt::format, iter::zip};

use crate::{git::branch_name::{self, RoswaalOwnedGitBranchName}, language::test::{RoswaalTest, RoswaalTestCommand}, utils::sqlite::{sqlite_repeat, SqliteRepeat, RoswaalSqliteTransaction}};
use anyhow::Result;
use sqlx::{query, query_as, FromRow, Sqlite};

use super::{progress::RoswaalTestProgressErrorDescription, query::{RoswaalSearchTestsQuery, RoswaalTestNamesString}};

#[derive(Debug, PartialEq, Eq)]
pub struct RoswaalStoredTest {
    name: String,
    description: Option<String>,
    steps: Vec<RoswaalStoredTestCommand>,
    error: Option<RoswaalTestProgressErrorDescription>,
    unmerged_branch_name: Option<RoswaalOwnedGitBranchName>
}

impl RoswaalStoredTest {
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct RoswaalStoredTestCommand {
    command: RoswaalTestCommand,
    did_pass: bool
}

impl <'a> RoswaalSqliteTransaction<'a> {
    pub async fn close_remove_tests_branch(
        &mut self,
        branch_name: &RoswaalOwnedGitBranchName
    ) -> Result<()> {
        query::<Sqlite>(statements::DELETE_STAGED_TEST_REMOVALS_WITH_BRANCH)
            .bind(branch_name)
            .execute(self.connection())
            .await?;
        Ok(())
    }

    pub async fn close_add_tests_branch(
        &mut self,
        branch_name: &RoswaalOwnedGitBranchName
    ) -> Result<()> {
        query::<Sqlite>(statements::DELETE_UNMERGED_TESTS_WITH_BRANCH)
            .bind(branch_name)
            .execute(self.connection())
            .await?;
        Ok(())
    }

    pub async fn stage_test_removals(
        &mut self,
        test_names: &RoswaalTestNamesString<'_>,
        branch_name: &RoswaalOwnedGitBranchName
    ) -> Result<()> {
        if test_names.is_empty() { return Ok(()) }
        sqlite_repeat(statements::INSERT_STAGED_TEST_REMOVAL, &test_names.iter().collect())
            .bind_to_query(|q, name| Ok(q.bind(name).bind(branch_name)))?
            .execute(self.connection())
            .await?;
        Ok(())
    }

    pub async fn merge_test_removals(
        &mut self,
        branch_name: &RoswaalOwnedGitBranchName
    ) -> Result<()> {
        let test_names = query_as::<Sqlite, SqliteTestName>(
            statements::SELECT_STAGED_TEST_REMOVAL_NAMES
        )
        .bind(branch_name)
        .fetch_all(self.connection())
        .await?;
        let delete_tests_statement = statements::delete_tests(test_names.iter().count());
        let mut delete_query = query::<Sqlite>(&delete_tests_statement);
        for sqlite_name in test_names.iter() {
            delete_query = delete_query.bind(&sqlite_name.name);
        }
        delete_query.execute(self.connection()).await?;
        self.close_remove_tests_branch(branch_name).await?;
        Ok(())
    }

    pub async fn merge_unmerged_tests(&mut self, branch_name: &RoswaalOwnedGitBranchName) -> Result<()> {
        let sqlite_location_names = query_as::<Sqlite, SqliteTestName>(
            statements::SELECT_UNMERGED_TEST_NAMES
        )
        .bind(branch_name)
        .fetch_all(self.connection())
        .await?;
        sqlite_repeat(statements::MERGE_UNMERGED_TESTS, &sqlite_location_names)
            .bind_to_query(|q, sqlite_name| {
                Ok(
                    q.bind(sqlite_name.name.clone())
                        .bind(branch_name)
                        .bind(sqlite_name.name.clone())
                )
            })?
            .execute(self.connection())
            .await?;
        Ok(())
    }

    pub async fn save_tests(
        &mut self,
        tests: &Vec<RoswaalTest>,
        branch_name: &RoswaalOwnedGitBranchName
    ) -> Result<()> {
        let mut tests = tests.clone();
        tests.reverse(); // NB: Ensure the last occurrence of each test is kept when dedupping.
        tests.dedup_by(|a, b| a.name() == b.name());
        let id_rows = sqlite_repeat(statements::INSERT_TEST_RETURNING_ID, &tests)
            .bind_to_query_as::<SqliteTestID>(|q, test| {
                Ok(q.bind(test.name()).bind(test.description()).bind(branch_name))
            })?
            .fetch_all(self.connection())
            .await?;
        sqlite_repeat(
            statements::INSERT_TEST_STEP,
            &(0..tests.iter().flat_map(|t| t.commands()).count()).collect()
        )
        .bind_custom_values_to_query(zip(tests.iter(), id_rows.iter()), |mut q, (test, id_row)| {
            for (ordinal, command) in test.commands().iter().enumerate() {
                q = q.bind(id_row.id).bind(serde_json::to_string(&command)?).bind(ordinal as i32)
            }
            Ok(q)
        })?
        .execute(self.connection())
        .await?;
        Ok(())
    }

    pub async fn tests_in_alphabetical_order(&mut self, query: &RoswaalSearchTestsQuery<'_>) -> Result<Vec<RoswaalStoredTest>> {
        let sqlite_tests = match query {
            RoswaalSearchTestsQuery::TestNames(test_names) => {
                let query_str = statements::select_tests_in_alphabetical_order(
                    test_names.iter().count()
                );
                let mut select_query = query_as::<Sqlite, SqliteStoredTestRow>(&query_str);
                for name in test_names.iter() {
                    select_query = select_query.bind(name.to_lowercase());
                }
                select_query.fetch_all(self.connection()).await?
            },
            RoswaalSearchTestsQuery::AllTests => {
                query_as::<Sqlite, SqliteStoredTestRow>(
                    statements::SELECT_ALL_TESTS_IN_ALPHABETICAL_ORDER
                )
                .fetch_all(self.connection())
                .await?
            }
        };
        if sqlite_tests.is_empty() { return Ok(vec![]) }
        let mut test = RoswaalStoredTest {
            name: sqlite_tests[0].test_name.clone(),
            description: sqlite_tests[0].description.clone(),
            steps: vec![],
            unmerged_branch_name: sqlite_tests[0].unmerged_branch_name.clone(),
            error: None
        };
        let mut tests = Vec::<RoswaalStoredTest>::new();
        for sqlite_test in sqlite_tests {
            let command = serde_json::from_str::<RoswaalTestCommand>(&sqlite_test.command_content)?;
            if sqlite_test.is_separate_from(&test) {
                tests.push(test);
                test = RoswaalStoredTest {
                    name: sqlite_test.test_name.clone(),
                    description: sqlite_test.description.clone(),
                    steps: vec![RoswaalStoredTestCommand { command, did_pass: sqlite_test.did_pass }],
                    unmerged_branch_name: sqlite_test.unmerged_branch_name.clone(),
                    error: None
                };
            } else {
                test.steps.push(RoswaalStoredTestCommand { command, did_pass: sqlite_test.did_pass })
            };
        }
        tests.push(test);
        Ok(tests)
    }
}

mod statements {
    use crate::utils::sqlite::sqlite_array_fields;

    pub const INSERT_STAGED_TEST_REMOVAL: &str = "
INSERT INTO StagedTestRemovals (
    name,
    unmerged_branch_name
) VALUES (
    LOWER(?),
    ?
) ON CONFLICT (name, unmerged_branch_name) DO NOTHING;
";

    pub const SELECT_ALL_TESTS_IN_ALPHABETICAL_ORDER: &str = "
SELECT
    t.name AS test_name,
    t.description,
    t.unmerged_branch_name,
    c.content AS command_content,
    c.did_pass
FROM Tests t
INNER JOIN TestSteps c ON t.id = c.test_id
ORDER BY test_name, c.ordinal;
";

    pub const MERGE_UNMERGED_TESTS: &str = "
DELETE FROM Tests WHERE name = ? AND unmerged_branch_name IS NULL;
UPDATE Tests SET unmerged_branch_name = NULL WHERE unmerged_branch_name = ? AND name = ?;
";

    pub const SELECT_UNMERGED_TEST_NAMES: &str =
        "SELECT name FROM Tests WHERE unmerged_branch_name = ?;";

    pub const SELECT_STAGED_TEST_REMOVAL_NAMES: &str =
        "SELECT name FROM StagedTestRemovals WHERE unmerged_branch_name = ?";

    pub const INSERT_TEST_STEP: &str =
        "INSERT INTO TestSteps (test_id, content, ordinal) VALUES (?, ?, ?);";

    pub const INSERT_TEST_RETURNING_ID: &str = "\
INSERT OR REPLACE INTO Tests (
    name,
    description,
    unmerged_branch_name
) VALUES (
    ?,
    ?,
    ?
) RETURNING id;";

    pub const DELETE_STAGED_TEST_REMOVALS_WITH_BRANCH: &str =
        "DELETE FROM StagedTestRemovals WHERE unmerged_branch_name = ?";

    pub const DELETE_UNMERGED_TESTS_WITH_BRANCH: &str =
        "DELETE FROM Tests WHERE unmerged_branch_name = ?";

    pub fn select_tests_in_alphabetical_order(count: usize) -> String {
        format!("
    SELECT
        t.name AS test_name,
        t.description,
        t.unmerged_branch_name,
        c.content AS command_content,
        c.did_pass
    FROM Tests t
    INNER JOIN TestSteps c ON t.id = c.test_id
    WHERE LOWER(test_name) IN {}
    ORDER BY test_name, c.ordinal;
    ", sqlite_array_fields(count))
    }

    pub fn delete_tests(count: usize) -> String {
        format!("\
DELETE FROM Tests
WHERE LOWER(name) IN {} AND unmerged_branch_name IS NULL
", sqlite_array_fields(count))
    }
}

#[derive(Debug, FromRow)]
struct SqliteTestName {
    name: String
}

#[derive(Debug, FromRow, Clone)]
struct SqliteTestID {
    id: i32
}

#[derive(Debug, FromRow)]
struct SqliteStoredTestRow {
    test_name: String,
    description: Option<String>,
    unmerged_branch_name: Option<RoswaalOwnedGitBranchName>,
    command_content: String,
    did_pass: bool
}

impl SqliteStoredTestRow {
    fn is_separate_from(&self, test: &RoswaalStoredTest) -> bool {
        test.name != self.test_name || test.unmerged_branch_name != self.unmerged_branch_name
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::{git::branch_name::{self, RoswaalOwnedGitBranchName}, language::test::{RoswaalTest, RoswaalTestCommand}, location::name::RoswaalLocationName, utils::sqlite::RoswaalSqlite};

    #[tokio::test]
    async fn test_store_and_retrieve_unmerged_tests() {
        let branch_name = RoswaalOwnedGitBranchName::new("test");
        let sqlite = RoswaalSqlite::in_memory().await.unwrap();
        let mut transaction = sqlite.transaction().await.unwrap();
        let tests = vec![
            RoswaalTest::new(
                "Test 1".to_string(),
                None,
                vec![
                    RoswaalTestCommand::Step {
                        name: "Step 1".to_string(),
                        requirement: "Requirement 1".to_string()
                    },
                    RoswaalTestCommand::SetLocation {
                        location_name: RoswaalLocationName::from_str("test").unwrap()
                    }
                ]
            ),
            RoswaalTest::new(
                "Test 2".to_string(),
                None,
                vec![
                    RoswaalTestCommand::Step {
                        name: "Step A".to_string(),
                        requirement: "Requirement A".to_string()
                    }
                ]
            )
        ];
        transaction.save_tests(&tests, &branch_name).await.unwrap();
        let stored_tests = transaction.tests_in_alphabetical_order(
            &RoswaalSearchTestsQuery::AllTests
        ).await.unwrap();
        let expected_tests = vec![
            RoswaalStoredTest {
                name: "Test 1".to_string(),
                description: None,
                steps: vec![
                    RoswaalStoredTestCommand {
                        command: RoswaalTestCommand::Step {
                            name: "Step 1".to_string(),
                            requirement: "Requirement 1".to_string()
                        },
                        did_pass: false
                    },
                    RoswaalStoredTestCommand {
                        command: RoswaalTestCommand::SetLocation {
                            location_name: RoswaalLocationName::from_str("test").unwrap()
                        },
                        did_pass: false
                    }
                ],
                error: None,
                unmerged_branch_name: Some(branch_name.clone())
            },
            RoswaalStoredTest {
                name: "Test 2".to_string(),
                description: None,
                steps: vec![
                    RoswaalStoredTestCommand {
                        command: RoswaalTestCommand::Step {
                            name: "Step A".to_string(),
                            requirement: "Requirement A".to_string()
                        },
                        did_pass: false
                    }
                ],
                error: None,
                unmerged_branch_name: Some(branch_name.clone())
            }
        ];
        assert_eq!(stored_tests, expected_tests)
    }

    #[tokio::test]
    async fn test_store_duplicate_named_tests_on_same_branch_replaces_initially_inserted_test() {
        let branch_name = RoswaalOwnedGitBranchName::new("test");
        let sqlite = RoswaalSqlite::in_memory().await.unwrap();
        let mut transaction = sqlite.transaction().await.unwrap();
        let tests = vec![
            RoswaalTest::new(
                "Test".to_string(),
                None,
                vec![
                    RoswaalTestCommand::Step {
                        name: "Step 1".to_string(),
                        requirement: "Requirement 1".to_string()
                    },
                    RoswaalTestCommand::SetLocation {
                        location_name: RoswaalLocationName::from_str("test").unwrap()
                    }
                ]
            ),
            RoswaalTest::new(
                "Test".to_string(),
                None,
                vec![
                    RoswaalTestCommand::Step {
                        name: "Step A".to_string(),
                        requirement: "Requirement A".to_string()
                    }
                ]
            )
        ];
        transaction.save_tests(&tests, &branch_name).await.unwrap();
        let stored_tests = transaction.tests_in_alphabetical_order(
            &RoswaalSearchTestsQuery::AllTests
        ).await.unwrap();
        let expected_tests = vec![
            RoswaalStoredTest {
                name: "Test".to_string(),
                description: None,
                steps: vec![
                    RoswaalStoredTestCommand {
                        command: RoswaalTestCommand::Step {
                            name: "Step A".to_string(),
                            requirement: "Requirement A".to_string()
                        },
                        did_pass: false
                    }
                ],
                error: None,
                unmerged_branch_name: Some(branch_name.clone())
            }
        ];
        assert_eq!(stored_tests, expected_tests)
    }

    #[tokio::test]
    async fn test_store_and_retrieve_unmerged_tests_with_same_name_and_different_branches() {
        let branch_name1 = RoswaalOwnedGitBranchName::new("test");
        let branch_name2 = RoswaalOwnedGitBranchName::new("test");
        let sqlite = RoswaalSqlite::in_memory().await.unwrap();
        let mut transaction = sqlite.transaction().await.unwrap();
        let mut tests = vec![
            RoswaalTest::new(
                "Test".to_string(),
                None,
                vec![
                    RoswaalTestCommand::Step {
                        name: "Step 1".to_string(),
                        requirement: "Requirement 1".to_string()
                    }
                ]
            )
        ];
        transaction.save_tests(&tests, &branch_name1).await.unwrap();
        tests = vec![
            RoswaalTest::new(
                "Test".to_string(),
                None,
                vec![
                    RoswaalTestCommand::Step {
                        name: "Step A".to_string(),
                        requirement: "Requirement A".to_string()
                    }
                ]
            )
        ];
        transaction.save_tests(&tests, &branch_name2).await.unwrap();
        let stored_tests = transaction.tests_in_alphabetical_order(
            &RoswaalSearchTestsQuery::AllTests
        ).await.unwrap();
        let expected_tests = vec![
            RoswaalStoredTest {
                name: "Test".to_string(),
                description: None,
                steps: vec![
                    RoswaalStoredTestCommand {
                        command: RoswaalTestCommand::Step {
                            name: "Step 1".to_string(),
                            requirement: "Requirement 1".to_string()
                        },
                        did_pass: false
                    }
                ],
                error: None,
                unmerged_branch_name: Some(branch_name1.clone())
            },
            RoswaalStoredTest {
                name: "Test".to_string(),
                description: None,
                steps: vec![
                    RoswaalStoredTestCommand {
                        command: RoswaalTestCommand::Step {
                            name: "Step A".to_string(),
                            requirement: "Requirement A".to_string()
                        },
                        did_pass: false
                    }
                ],
                error: None,
                unmerged_branch_name: Some(branch_name2.clone())
            }
        ];
        assert_eq!(stored_tests, expected_tests)
    }

    #[tokio::test]
    async fn test_returns_empty_vector_when_no_inserted_tests() {
        let sqlite = RoswaalSqlite::in_memory().await.unwrap();
        let mut transaction = sqlite.transaction().await.unwrap();
        let tests = transaction.tests_in_alphabetical_order(
            &RoswaalSearchTestsQuery::AllTests
        ).await.unwrap();
        assert_eq!(tests, vec![])
    }

    #[tokio::test]
    async fn test_store_and_merge_tests_removes_branch_name_of_merged_branch() {
        let branch_name = RoswaalOwnedGitBranchName::new("test");
        let sqlite = RoswaalSqlite::in_memory().await.unwrap();
        let mut transaction = sqlite.transaction().await.unwrap();
        let mut tests = vec![
            RoswaalTest::new(
                "Test 1".to_string(),
                None,
                vec![
                    RoswaalTestCommand::Step {
                        name: "Step 1".to_string(),
                        requirement: "Requirement 1".to_string()
                    },
                    RoswaalTestCommand::SetLocation {
                        location_name: RoswaalLocationName::from_str("test").unwrap()
                    }
                ]
            )
        ];
        transaction.save_tests(&tests, &branch_name).await.unwrap();
        let branch_name2 = RoswaalOwnedGitBranchName::new("test-2");
        tests = vec![
            RoswaalTest::new(
                "Test 2".to_string(),
                None,
                vec![
                    RoswaalTestCommand::Step {
                        name: "Step A".to_string(),
                        requirement: "Requirement A".to_string()
                    }
                ]
            )
        ];
        transaction.save_tests(&tests, &branch_name2).await.unwrap();
        transaction.merge_unmerged_tests(&branch_name).await.unwrap();
        let stored_tests_branch_names = transaction.tests_in_alphabetical_order(
            &RoswaalSearchTestsQuery::AllTests
        ).await.unwrap()
            .iter()
            .map(|t| t.unmerged_branch_name.clone())
            .collect::<Vec<Option<RoswaalOwnedGitBranchName>>>();
        assert_eq!(stored_tests_branch_names, vec![None, Some(branch_name2)])
    }

    #[tokio::test]
    async fn test_store_and_merge_tests_with_same_name_overwrites_previous_merged() {
        let branch_name = RoswaalOwnedGitBranchName::new("test");
        let sqlite = RoswaalSqlite::in_memory().await.unwrap();
        let mut transaction = sqlite.transaction().await.unwrap();
        let mut tests = vec![
            RoswaalTest::new(
                "Test".to_string(),
                None,
                vec![
                    RoswaalTestCommand::Step {
                        name: "Step 1".to_string(),
                        requirement: "Requirement 1".to_string()
                    },
                    RoswaalTestCommand::SetLocation {
                        location_name: RoswaalLocationName::from_str("test").unwrap()
                    }
                ]
            )
        ];
        transaction.save_tests(&tests, &branch_name).await.unwrap();
        transaction.merge_unmerged_tests(&branch_name).await.unwrap();
        let branch_name2 = RoswaalOwnedGitBranchName::new("test-2");
        tests = vec![
            RoswaalTest::new(
                "Test".to_string(),
                None,
                vec![
                    RoswaalTestCommand::Step {
                        name: "Step A".to_string(),
                        requirement: "Requirement A".to_string()
                    }
                ]
            )
        ];
        transaction.save_tests(&tests, &branch_name2).await.unwrap();
        transaction.merge_unmerged_tests(&branch_name2).await.unwrap();
        let stored_tests = transaction.tests_in_alphabetical_order(
            &RoswaalSearchTestsQuery::AllTests
        ).await.unwrap();
        let expected_tests = vec![
            RoswaalStoredTest {
                name: "Test".to_string(),
                description: None,
                steps: vec![
                    RoswaalStoredTestCommand {
                        command: RoswaalTestCommand::Step {
                            name: "Step A".to_string(),
                            requirement: "Requirement A".to_string()
                        },
                        did_pass: false
                    }
                ],
                error: None,
                unmerged_branch_name: None
            }
        ];
        assert_eq!(stored_tests, expected_tests)
    }

    #[tokio::test]
    async fn query_partial_tests() {
        let branch_name = RoswaalOwnedGitBranchName::new("test");
        let sqlite = RoswaalSqlite::in_memory().await.unwrap();
        let mut transaction = sqlite.transaction().await.unwrap();
        let tests = vec![
            RoswaalTest::new(
                "Dazai Is Insane".to_string(),
                None,
                vec![
                    RoswaalTestCommand::Step {
                        name: "Step 1".to_string(),
                        requirement: "Requirement 1".to_string()
                    },
                    RoswaalTestCommand::SetLocation {
                        location_name: RoswaalLocationName::from_str("test").unwrap()
                    }
                ]
            ),
            RoswaalTest::new(
                "Zanza The Divine".to_string(),
                None,
                vec![
                    RoswaalTestCommand::Step {
                        name: "Step A".to_string(),
                        requirement: "Requirement A".to_string()
                    }
                ]
            ),
            RoswaalTest::new(
                "L".to_string(),
                None,
                vec![
                    RoswaalTestCommand::Step {
                        name: "Step B".to_string(),
                        requirement: "Requirement C".to_string()
                    }
                ]
            )
        ];
        transaction.save_tests(&tests, &branch_name).await.unwrap();
        transaction.merge_unmerged_tests(&branch_name).await.unwrap();
        let query_str = "\
Zanza The Divine
l
";
        let stored_tests = transaction.tests_in_alphabetical_order(
            &RoswaalSearchTestsQuery::new(query_str)
        ).await.unwrap();
        let expected_test_names = vec!["L", "Zanza The Divine"];
        assert_eq!(stored_tests.iter().map(|t| t.name()).collect::<Vec<&str>>(), expected_test_names)
    }

    #[tokio::test]
    async fn stage_test_removals_does_not_remove_tests() {
        let mut branch_name = RoswaalOwnedGitBranchName::new("test");
        let sqlite = RoswaalSqlite::in_memory().await.unwrap();
        let mut transaction = sqlite.transaction().await.unwrap();
        let tests = vec![
            RoswaalTest::new(
                "Zanza The Divine".to_string(),
                None,
                vec![
                    RoswaalTestCommand::Step {
                        name: "Step A".to_string(),
                        requirement: "Requirement A".to_string()
                    }
                ]
            )
        ];
        transaction.save_tests(&tests, &branch_name).await.unwrap();
        transaction.merge_unmerged_tests(&branch_name).await.unwrap();
        let names_str = "Zanza The Divine";
        branch_name = RoswaalOwnedGitBranchName::new("stage");
        transaction.stage_test_removals(
            &RoswaalTestNamesString(names_str),
            &branch_name
        ).await.unwrap();
        let stored_tests = transaction.tests_in_alphabetical_order(
            &RoswaalSearchTestsQuery::new(names_str)
        ).await.unwrap();
        let expected_test_names = vec!["Zanza The Divine"];
        assert_eq!(stored_tests.iter().map(|t| t.name()).collect::<Vec<&str>>(), expected_test_names)
    }

    #[tokio::test]
    async fn remove_merged_tests_only_removes_merged_tests() {
        let branch_name = RoswaalOwnedGitBranchName::new("test");
        let sqlite = RoswaalSqlite::in_memory().await.unwrap();
        let mut transaction = sqlite.transaction().await.unwrap();
        let mut tests = vec![
            RoswaalTest::new(
                "Dazai Is Insane".to_string(),
                None,
                vec![
                    RoswaalTestCommand::Step {
                        name: "Step 1".to_string(),
                        requirement: "Requirement 1".to_string()
                    },
                    RoswaalTestCommand::SetLocation {
                        location_name: RoswaalLocationName::from_str("test").unwrap()
                    }
                ]
            ),
            RoswaalTest::new(
                "Zanza The Divine".to_string(),
                None,
                vec![
                    RoswaalTestCommand::Step {
                        name: "Step A".to_string(),
                        requirement: "Requirement A".to_string()
                    }
                ]
            )
        ];
        transaction.save_tests(&tests, &branch_name).await.unwrap();
        transaction.merge_unmerged_tests(&branch_name).await.unwrap();
        let branch_name2 = RoswaalOwnedGitBranchName::new("test-2");
        tests = vec![
            RoswaalTest::new(
                "L".to_string(),
                None,
                vec![
                    RoswaalTestCommand::Step {
                        name: "Step B".to_string(),
                        requirement: "Requirement C".to_string()
                    }
                ]
            )
        ];
        transaction.save_tests(&tests, &branch_name2).await.unwrap();
        let names_str = "\
Zanza The Divine
L
";
        let stage_branch = RoswaalOwnedGitBranchName::new("stage");
        transaction.stage_test_removals(
            &RoswaalTestNamesString(names_str),
            &stage_branch
        ).await.unwrap();
        transaction.merge_test_removals(&stage_branch).await.unwrap();
        let stored_tests = transaction.tests_in_alphabetical_order(
            &RoswaalSearchTestsQuery::new(names_str)
        ).await.unwrap();
        let expected_test_names = vec!["L"];
        assert_eq!(stored_tests.iter().map(|t| t.name()).collect::<Vec<&str>>(), expected_test_names)
    }

    #[tokio::test]
    async fn close_add_branch_removes_unmerged_tests_on_branch() {
        let mut branch_name = RoswaalOwnedGitBranchName::new("test");
        let sqlite = RoswaalSqlite::in_memory().await.unwrap();
        let mut transaction = sqlite.transaction().await.unwrap();
        let mut tests = vec![
            RoswaalTest::new(
                "Zanza The Divine".to_string(),
                None,
                vec![
                    RoswaalTestCommand::Step {
                        name: "Step A".to_string(),
                        requirement: "Requirement A".to_string()
                    }
                ]
            )
        ];
        transaction.save_tests(&tests, &branch_name).await.unwrap();
        tests = vec![
            RoswaalTest::new(
                "L".to_string(),
                None,
                vec![
                    RoswaalTestCommand::Step {
                        name: "Step A".to_string(),
                        requirement: "Requirement A".to_string()
                    }
                ]
            )
        ];
        branch_name = RoswaalOwnedGitBranchName::new("test-2");
        transaction.save_tests(&tests, &branch_name).await.unwrap();
        transaction.close_add_tests_branch(&branch_name).await.unwrap();
        let stored_tests = transaction.tests_in_alphabetical_order(
            &RoswaalSearchTestsQuery::AllTests
        ).await.unwrap();
        let expected_test_names = vec!["Zanza The Divine"];
        assert_eq!(stored_tests.iter().map(|t| t.name()).collect::<Vec<&str>>(), expected_test_names)
    }

    #[tokio::test]
    async fn close_remove_branch_removes_staged_test_removals() {
        let mut branch_name = RoswaalOwnedGitBranchName::new("test");
        let sqlite = RoswaalSqlite::in_memory().await.unwrap();
        let mut transaction = sqlite.transaction().await.unwrap();
        let mut tests = vec![
            RoswaalTest::new(
                "Zanza The Divine".to_string(),
                None,
                vec![
                    RoswaalTestCommand::Step {
                        name: "Step A".to_string(),
                        requirement: "Requirement A".to_string()
                    }
                ]
            )
        ];
        transaction.save_tests(&tests, &branch_name).await.unwrap();
        transaction.merge_unmerged_tests(&branch_name).await.unwrap();
        branch_name = RoswaalOwnedGitBranchName::new("removal");
        transaction.stage_test_removals(
            &RoswaalTestNamesString("Zanza The Divine"),
            &branch_name
        ).await.unwrap();
        transaction.close_remove_tests_branch(&branch_name).await.unwrap();
        tests = vec![
            RoswaalTest::new(
                "L".to_string(),
                None,
                vec![
                    RoswaalTestCommand::Step {
                        name: "Step A".to_string(),
                        requirement: "Requirement A".to_string()
                    }
                ]
            )
        ];
        branch_name = RoswaalOwnedGitBranchName::new("test-2");
        transaction.save_tests(&tests, &branch_name).await.unwrap();
        transaction.close_add_tests_branch(&branch_name).await.unwrap();
        let stored_tests = transaction.tests_in_alphabetical_order(
            &RoswaalSearchTestsQuery::AllTests
        ).await.unwrap();
        let expected_test_names = vec!["Zanza The Divine"];
        assert_eq!(stored_tests.iter().map(|t| t.name()).collect::<Vec<&str>>(), expected_test_names)
    }
}
