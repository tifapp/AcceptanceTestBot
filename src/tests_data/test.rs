use chrono::{DateTime, Utc};

use crate::{
    git::branch_name::RoswaalOwnedGitBranchName, language::test::RoswaalCompiledTestCommand,
};

use super::ordinal::RoswaalTestCommandOrdinal;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RoswaalTest {
    name: String,
    description: Option<String>,
    commands: Vec<RoswaalCompiledTestCommand>,
    command_failure_ordinal: Option<RoswaalTestCommandOrdinal>,
    error_message: Option<String>,
    error_stack_trace: Option<String>,
    unmerged_branch_name: Option<RoswaalOwnedGitBranchName>,
    last_run_date: Option<DateTime<Utc>>,
}

impl RoswaalTest {
    pub fn new(
        name: String,
        description: Option<String>,
        commands: Vec<RoswaalCompiledTestCommand>,
        command_failure_ordinal: Option<RoswaalTestCommandOrdinal>,
        error_message: Option<String>,
        error_stack_trace: Option<String>,
        unmerged_branch_name: Option<RoswaalOwnedGitBranchName>,
        last_run_date: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            name,
            description,
            commands,
            command_failure_ordinal,
            error_message,
            error_stack_trace,
            unmerged_branch_name,
            last_run_date,
        }
    }
}

impl RoswaalTest {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&String> {
        self.description.as_ref()
    }

    pub fn command_failure_ordinal(&self) -> Option<RoswaalTestCommandOrdinal> {
        self.command_failure_ordinal
    }

    pub fn commands(&self) -> Vec<RoswaalTestCommand> {
        self.commands
            .iter()
            .enumerate()
            .map(|(i, c)| RoswaalTestCommand {
                status: self.command_status(RoswaalTestCommandOrdinal::new(i as i32)),
                command: c.clone(),
            })
            .collect()
    }

    pub fn command_status(&self, ordinal: RoswaalTestCommandOrdinal) -> RoswaalTestCommandStatus {
        match (self.last_run_date(), self.command_failure_ordinal()) {
            (None, _) => RoswaalTestCommandStatus::Idle,
            (Some(_), None) => RoswaalTestCommandStatus::Passed,
            (Some(_), Some(failure_ordinal)) => {
                if ordinal < failure_ordinal {
                    RoswaalTestCommandStatus::Passed
                } else {
                    RoswaalTestCommandStatus::Failed
                }
            }
        }
    }

    pub fn error_message(&self) -> Option<&String> {
        self.error_message.as_ref()
    }

    pub fn error_stack_trace(&self) -> Option<&String> {
        self.error_stack_trace.as_ref()
    }

    pub fn last_run_date(&self) -> Option<DateTime<Utc>> {
        self.last_run_date
    }

    pub fn unmerged_branch_name(&self) -> Option<&RoswaalOwnedGitBranchName> {
        self.unmerged_branch_name.as_ref()
    }

    pub(super) fn push_compiled_command(&mut self, command: RoswaalCompiledTestCommand) {
        self.commands.push(command)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoswaalTestCommand {
    status: RoswaalTestCommandStatus,
    command: RoswaalCompiledTestCommand,
}

impl RoswaalTestCommand {
    pub fn status(&self) -> RoswaalTestCommandStatus {
        self.status
    }

    pub fn command(&self) -> &RoswaalCompiledTestCommand {
        &self.command
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RoswaalTestCommandStatus {
    Passed,
    Failed,
    Idle,
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::{
        language::test::RoswaalCompiledTestCommand,
        tests_data::{
            ordinal::RoswaalTestCommandOrdinal,
            test::{RoswaalTestCommand, RoswaalTestCommandStatus},
        },
    };

    use super::RoswaalTest;

    #[test]
    fn command_status_on_ran_test() {
        let test = RoswaalTest::new(
            "Test".to_string(),
            None,
            vec![
                RoswaalCompiledTestCommand::Step {
                    label: "Step 1".to_string(),
                    name: "Thing".to_string(),
                    requirement: "Thing".to_string(),
                },
                RoswaalCompiledTestCommand::Step {
                    label: "Step 2".to_string(),
                    name: "Thing 2".to_string(),
                    requirement: "Thing 2".to_string(),
                },
            ],
            Some(RoswaalTestCommandOrdinal::new(1)),
            Some("WTF".to_string()),
            Some("Stack Trace".to_string()),
            None,
            Some(Utc::now()),
        );
        assert_eq!(
            test.command_status(RoswaalTestCommandOrdinal::for_before_launch()),
            RoswaalTestCommandStatus::Passed
        );
        assert_eq!(
            test.command_status(RoswaalTestCommandOrdinal::new(0)),
            RoswaalTestCommandStatus::Passed
        );
        assert_eq!(
            test.command_status(RoswaalTestCommandOrdinal::new(1)),
            RoswaalTestCommandStatus::Failed
        )
    }

    #[test]
    fn command_status_on_idle_test() {
        let test = RoswaalTest::new(
            "Test".to_string(),
            None,
            vec![
                RoswaalCompiledTestCommand::Step {
                    label: "Step 1".to_string(),
                    name: "Thing".to_string(),
                    requirement: "Thing".to_string(),
                },
                RoswaalCompiledTestCommand::Step {
                    label: "Step 2".to_string(),
                    name: "Thing 2".to_string(),
                    requirement: "Thing 2".to_string(),
                },
            ],
            None,
            None,
            None,
            None,
            None,
        );
        assert_eq!(
            test.command_status(RoswaalTestCommandOrdinal::for_before_launch()),
            RoswaalTestCommandStatus::Idle
        );
        assert_eq!(
            test.command_status(RoswaalTestCommandOrdinal::new(0)),
            RoswaalTestCommandStatus::Idle
        );
        assert_eq!(
            test.command_status(RoswaalTestCommandOrdinal::new(1)),
            RoswaalTestCommandStatus::Idle
        )
    }

    #[test]
    fn commands() {
        let test = RoswaalTest::new(
            "Test".to_string(),
            None,
            vec![
                RoswaalCompiledTestCommand::Step {
                    label: "Step 1".to_string(),
                    name: "Thing".to_string(),
                    requirement: "Thing".to_string(),
                },
                RoswaalCompiledTestCommand::Step {
                    label: "Step 2".to_string(),
                    name: "Thing 2".to_string(),
                    requirement: "Thing 2".to_string(),
                },
            ],
            Some(RoswaalTestCommandOrdinal::new(1)),
            Some("WTF".to_string()),
            Some("Stack Trace".to_string()),
            None,
            Some(Utc::now()),
        );
        let expected_commands = vec![
            RoswaalTestCommand {
                status: RoswaalTestCommandStatus::Passed,
                command: test.commands[0].clone(),
            },
            RoswaalTestCommand {
                status: RoswaalTestCommandStatus::Failed,
                command: test.commands[1].clone(),
            },
        ];
        assert_eq!(test.commands(), expected_commands)
    }
}
