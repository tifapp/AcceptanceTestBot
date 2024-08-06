use std::borrow::Borrow;

use crate::{
    language::test::RoswaalTestCommand,
    operations::search_tests::SearchTestsStatus,
    tests_data::storage::{RoswaalStoredTest, RoswaalStoredTestCommand},
};

use super::{
    branch_name_view::OptionalBranchNameView,
    ui_lib::{
        block_kit_views::{SlackDivider, SlackHeader, SlackSection},
        for_each_view::ForEachView,
        if_let_view::IfLet,
        if_view::If,
        slack_view::SlackView,
    },
};

pub struct SearchTestsView {
    status: SearchTestsStatus,
}

impl SearchTestsView {
    pub fn new(status: SearchTestsStatus) -> Self {
        Self { status }
    }
}

impl SlackView for SearchTestsView {
    fn slack_body(&self) -> impl SlackView {
        SlackHeader::new("Test Progress").flat_chain_block(self.status_view())
    }
}

impl SearchTestsView {
    fn status_view(&self) -> impl SlackView {
        match self.status.borrow() {
            SearchTestsStatus::Success(tests) => ForEachView::new(
                tests.iter().map(|t| t.clone()).enumerate(),
                |(index, test)| {
                    TestView { test: test.clone() }
                        .flat_chain_block(If::is_true(*index < tests.len() - 1, || SlackDivider))
                },
            )
            .erase_to_any_view(),
            SearchTestsStatus::NoTests => {
                SlackSection::from_markdown("🔴 No tests were fooooooound.").erase_to_any_view()
            }
        }
    }
}

struct TestView {
    test: RoswaalStoredTest,
}

impl SlackView for TestView {
    fn slack_body(&self) -> impl SlackView {
        SlackSection::from_markdown(&format!("📝 *{}*", self.test.name()))
            .flat_chain_block(IfLet::some(self.test.description(), |text| {
                SlackSection::from_plaintext(text)
            }))
            .flat_chain_block(match self.test.last_run_date() {
                Some(date) => {
                    let formatted_date = date.format("%Y-%m-%d %H:%M:%S").to_string();
                    let message = format!("_Last Ran: {}_", formatted_date);
                    SlackSection::from_markdown(&message)
                }
                None => SlackSection::from_markdown("_This test has never been run._"),
            })
            .flat_chain_block(SlackSection::from_markdown(&format!(
                "{} *Before Launch*",
                status_emoji(self.test.did_pass_before_launch())
            )))
            .flat_chain_block(ForEachView::new(
                self.test.commands().iter().map(|e| e.clone()),
                |stored_command| CommandView {
                    stored_command: stored_command.clone(),
                },
            ))
            .flat_chain_block(IfLet::some(self.test.error_message(), |message| {
                let message = format!("⚠️ *Error Message*\n{}", message);
                SlackSection::from_markdown(&message)
            }))
            .flat_chain_block(IfLet::some(self.test.error_stack_trace(), |stack_trace| {
                let stack_trace = format!("⚠️ *Stack Trace*\n{}", stack_trace);
                SlackSection::from_markdown(&stack_trace)
            }))
            .flat_chain_block(OptionalBranchNameView::new(
                self.test.unmerged_branch_name(),
            ))
    }
}

struct CommandView {
    stored_command: RoswaalStoredTestCommand,
}

impl SlackView for CommandView {
    fn slack_body(&self) -> impl SlackView {
        match self.stored_command.command() {
            RoswaalTestCommand::Step {
                label,
                name,
                requirement,
            } => {
                let body = format!(
                    "{} *{}:* {} _({})_\n",
                    self.status_emoji(),
                    label,
                    name,
                    requirement
                );
                SlackSection::from_markdown(&body)
            }
            RoswaalTestCommand::SetLocation { location_name } => {
                let body = format!(
                    "{} *Set Location:* {}\n",
                    self.status_emoji(),
                    location_name.raw_name()
                );
                SlackSection::from_markdown(&body)
            }
        }
    }
}

impl CommandView {
    fn status_emoji(&self) -> &'static str {
        status_emoji(self.stored_command.did_pass())
    }
}

fn status_emoji(is_passing: Option<bool>) -> &'static str {
    match is_passing {
        Some(true) => "✅",
        Some(false) => "🔴",
        None => "🔘",
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use chrono::{DateTime, NaiveDateTime, Utc};

    use crate::{
        language::{ast::RoswaalTestSyntaxLine, test::RoswaalTestCommand},
        location::name::RoswaalLocationName,
        operations::search_tests::SearchTestsStatus,
        slack::{
            test_support::SlackTestConstantBranches,
            ui_lib::test_support::{assert_slack_view_snapshot, SnapshotMode},
        },
        tests_data::{
            ordinal::RoswaalTestCommandOrdinal, progress::RoswaalTestProgressErrorDescription,
            storage::RoswaalStoredTest,
        },
    };

    use super::SearchTestsView;

    #[test]
    fn success_snapshot() {
        let branches = SlackTestConstantBranches::load();
        let date = "2024-07-24T00:00:00+0000".parse::<DateTime<Utc>>().unwrap();
        let tests = vec![
            RoswaalStoredTest::new(
                "Test Idle".to_string(),
                None,
                vec![RoswaalTestCommand::Step {
                    label: "Step A".to_string(),
                    name: "Do the thing".to_string(),
                    requirement: "Do the thing".to_string(),
                }],
                None,
                None,
                None,
                None,
            ),
            RoswaalStoredTest::new(
                "Test Unmerged".to_string(),
                None,
                vec![RoswaalTestCommand::Step {
                    label: "Step A".to_string(),
                    name: "Do the thing".to_string(),
                    requirement: "Do the thing".to_string(),
                }],
                None,
                None,
                Some(branches.add_tests().clone()),
                None,
            ),
            RoswaalStoredTest::new(
                "Test Passing".to_string(),
                Some("I am the fucking strong".to_string()),
                vec![
                    RoswaalTestCommand::SetLocation {
                        location_name: RoswaalLocationName::from_str("Oakland").unwrap(),
                    },
                    RoswaalTestCommand::Step {
                        label: "Step 1".to_string(),
                        name: "Do the thing".to_string(),
                        requirement: "Do the thing".to_string(),
                    },
                ],
                None,
                None,
                None,
                Some(date),
            ),
            RoswaalStoredTest::new(
                "Test Failing Before Launch".to_string(),
                None,
                vec![
                    RoswaalTestCommand::SetLocation {
                        location_name: RoswaalLocationName::from_str("Oakland").unwrap(),
                    },
                    RoswaalTestCommand::Step {
                        label: "Step 1".to_string(),
                        name: "Do the thing".to_string(),
                        requirement: "Do the thing".to_string(),
                    },
                ],
                Some(RoswaalTestCommandOrdinal::for_before_launch()),
                Some(RoswaalTestProgressErrorDescription::new(
                    "Everyone Died".to_string(),
                    "Lol figure it out yourself...".to_string(),
                )),
                None,
                Some(date),
            ),
            RoswaalStoredTest::new(
                "Test Failing After Launch".to_string(),
                None,
                vec![
                    RoswaalTestCommand::SetLocation {
                        location_name: RoswaalLocationName::from_str("San Jose").unwrap(),
                    },
                    RoswaalTestCommand::Step {
                        label: "Step 1".to_string(),
                        name: "Do the thing".to_string(),
                        requirement: "Do the thing".to_string(),
                    },
                    RoswaalTestCommand::Step {
                        label: "Step 2".to_string(),
                        name: "I am the fucking strong".to_string(),
                        requirement: "So that's what I'll do".to_string(),
                    },
                ],
                Some(RoswaalTestCommandOrdinal::new(1)),
                Some(RoswaalTestProgressErrorDescription::new(
                    "HAHAHHAHAHAHAHHAHAHAHAHAHAHHHAAHHAHAH".to_string(),
                    "GLHF".to_string(),
                )),
                None,
                Some(date),
            ),
        ];
        assert_slack_view_snapshot(
            "search-tests-success",
            &SearchTestsView::new(SearchTestsStatus::Success(tests)),
            SnapshotMode::Comparing,
        )
    }

    #[test]
    fn no_tests_found_snapshot() {
        assert_slack_view_snapshot(
            "search-tests-no-tests-found",
            &SearchTestsView::new(SearchTestsStatus::NoTests),
            SnapshotMode::Comparing,
        )
    }
}
