use std::iter::zip;

use crate::{git::branch_name::RoswaalOwnedGitBranchName, language::test::{RoswaalTest, RoswaalTestCommand}, utils::sqlite::RoswaalSqliteTransaction};
use anyhow::Result;
use sqlx::{query, query_as, FromRow, Sqlite};

use super::progress::RoswaalTestProgressErrorDescription;

#[derive(Debug, PartialEq, Eq)]
pub struct RoswaalStoredTest {
    name: String,
    description: Option<String>,
    steps: Vec<RoswaalStoredTestCommand>,
    error: Option<RoswaalTestProgressErrorDescription>,
    unmerged_branch_name: Option<RoswaalOwnedGitBranchName>
}

#[derive(Debug, PartialEq, Eq)]
pub struct RoswaalStoredTestCommand {
    command: RoswaalTestCommand,
    did_pass: bool
}

impl <'a> RoswaalSqliteTransaction<'a> {
    async fn save_tests(
        &mut self,
        tests: &Vec<RoswaalTest>,
        branch_name: &RoswaalOwnedGitBranchName
    ) -> Result<()> {
        let statements = tests.iter().map(|_| {
            "INSERT INTO Tests (name, description, unmerged_branch_name) VALUES (?, ?, ?) RETURNING id;"
        })
        .collect::<Vec<&str>>()
        .join("\n");
        let mut tests_insert_query = query_as::<Sqlite, SqliteTestIDRow>(&statements);
        for test in tests.iter() {
            tests_insert_query = tests_insert_query.bind(test.name())
                .bind(test.description())
                .bind(branch_name.to_string());
        }
        let id_rows = tests_insert_query.fetch_all(self.connection()).await?;
        let command_insert_statements = tests.iter()
            .flat_map(|t| t.commands())
            .map(|_| {
                "INSERT INTO TestSteps (test_id, content, unmerged_branch_name) VALUES (?, ?, ?);"
            })
            .collect::<Vec<&str>>()
            .join("\n");
        let mut commands_insert_query = query::<Sqlite>(&command_insert_statements);
        for (test, id_row) in zip(tests.iter(), id_rows.iter()) {
            for command in test.commands() {
                commands_insert_query = commands_insert_query.bind(id_row.id)
                    .bind(serde_json::to_string(&command)?)
                    .bind(branch_name.to_string())
            }
        }
        commands_insert_query.execute(self.connection()).await?;
        Ok(())
    }

    async fn tests_in_alphabetical_order(&mut self) -> Result<Vec<RoswaalStoredTest>> {
        let sqlite_tests = query_as::<Sqlite, SqliteStoredTestRow>(SELECT_TESTS_IN_ALPHABETICAL_ORDER)
            .fetch_all(self.connection())
            .await?;
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

const SELECT_TESTS_IN_ALPHABETICAL_ORDER: &str = "
SELECT
    t.name AS test_name,
    t.description,
    t.unmerged_branch_name,
    c.content AS command_content,
    c.did_pass
FROM Tests t
INNER JOIN TestSteps c ON t.id = c.test_id
ORDER BY test_name;
";

#[derive(Debug, FromRow)]
struct SqliteTestIDRow {
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
        let stored_tests = transaction.tests_in_alphabetical_order().await.unwrap();
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
        let stored_tests = transaction.tests_in_alphabetical_order().await.unwrap();
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
        let tests = transaction.tests_in_alphabetical_order().await.unwrap();
        assert_eq!(tests, vec![])
    }
}
