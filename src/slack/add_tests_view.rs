use std::borrow::Borrow;

use crate::{language::{compiler::{RoswaalCompilationDuplicateErrorCode, RoswaalCompilationError, RoswaalCompilationErrorCode}, test::RoswaalTest}, location::name::RoswaalLocationNameParsingError, operations::add_tests::{AddTestsStatus, RoswaalTestCompilationResults}};

use super::{merge_conflict_view::MergeConflictView, pr_open_fail_view::FailedToOpenPullRequestView, ui_lib::{block_kit_views::{SlackDivider, SlackHeader, SlackSection}, empty_view::EmptySlackView, for_each_view::ForEachView, if_view::If, slack_view::SlackView}, users::MATTHEW_SLACK_USER_ID, warn_undeleted_branch_view::WarnUndeletedBranchView};

pub struct AddTestsView {
    status: AddTestsStatus
}

impl AddTestsView {
    pub fn new(status: AddTestsStatus) -> Self {
        Self { status }
    }
}

impl SlackView for AddTestsView {
    fn slack_body(&self) -> impl SlackView {
        SlackHeader::new("Add Tests")
            .flat_chain_block(self.status_view())
    }
}

impl AddTestsView {
    fn status_view(&self) -> impl SlackView {
        match self.status.borrow() {
            AddTestsStatus::Success { results, should_warn_undeleted_branch } => {
                If::is_true(
                    results.has_compiling_tests(),
                    || self.compiling_tests_view(&results.tests())
                )
                .flat_chain_block(
                    If::is_true(
                        results.has_non_compiling_tests(),
                        || self.non_compiling_tests_view(&results.errors())
                    )
                )
                .flat_chain_block(
                    If::is_true(
                        results.has_compiling_tests(),
                        || {
                            SlackDivider.flat_chain_block(
                                SlackHeader::new("Next Steps")
                                    .flat_chain_block(
                                        SlackSection::from_markdown(
                                            "Approve the PR found in <#C01B7FFKDCP> to finish the adding the teeeeeeeests!"
                                        )
                                    )
                            )
                        }
                    )
                )
                .flat_chain_block(
                    If::is_true(
                        *should_warn_undeleted_branch,
                        || SlackDivider.flat_chain_block(WarnUndeletedBranchView)
                    )
                )
                .erase_to_any_view()
            },
            AddTestsStatus::NoTestsFound => {
                SlackSection::from_markdown("🔴 No tests were fooooooound.")
                    .erase_to_any_view()
            },
            AddTestsStatus::MergeConflict => {
                MergeConflictView::new(MATTHEW_SLACK_USER_ID).erase_to_any_view()
            },
            AddTestsStatus::FailedToOpenPullRequest => {
                FailedToOpenPullRequestView.erase_to_any_view()
            },
        }
    }
}

impl AddTestsView {
    fn compiling_tests_view(&self, tests: &Vec<RoswaalTest>) -> impl SlackView {
        let mut body = "✅ *The following tests were compiled succeeeeeeeeessfully!*\n".to_string();
        for test in tests {
            body.push_str(&format!("- {}\n", test.name()))
        }
        SlackSection::from_markdown(&body)
    }

    fn non_compiling_tests_view(&self, tests: &Vec<(usize, Vec<RoswaalCompilationError>)>) -> impl SlackView {
        let tests_iter = tests.iter()
            .map(|e| e.clone())
            .enumerate()
            .map(|(index, value)| (index < tests.len() - 1, value));
        SlackSection::from_markdown(
            "⚠️ *The following tests did not compile succeeeeeessfully. They are only listed by naaaaaaaame!*"
        )
        .flat_chain_block(
            ForEachView::new(tests_iter, |(is_showing_divider, (test_number, errors))| {
                let test_number = (*test_number as u32) + 1;
                NonCompilingTestView { test_number, errors: errors.clone() }
                    .flat_chain_block(If::is_true(*is_showing_divider, || SlackDivider))
            })
        )
    }
}

struct NonCompilingTestView {
    test_number: u32,
    errors: Vec<RoswaalCompilationError>
}

impl SlackView for NonCompilingTestView {
    fn slack_body(&self) -> impl SlackView {
        SlackSection::from_markdown(&format!("❗️ *Test {}*", self.test_number))
            .flat_chain_block(
                ForEachView::new(self.errors.iter(), |error| CompilationErrorView { error })
            )
    }
}

struct CompilationErrorView<'v> {
    error: &'v RoswaalCompilationError
}

impl <'v> SlackView for CompilationErrorView<'v> {
    fn slack_body(&self) -> impl SlackView {
        let mut body = String::new();
        match self.error.code() {
            RoswaalCompilationErrorCode::NoTestName => {
                body.push_str("No test name was speeeeeeecified.")
            },
            RoswaalCompilationErrorCode::NoTestSteps => {
                body.push_str("No test steps were speeeeeeeecified.")
            },
            RoswaalCompilationErrorCode::NoCommandDescription { command_name } => {
                body.push_str(&format!("No command description was specified for \"{}\".", command_name))
            },
            RoswaalCompilationErrorCode::NoStepRequirement { step_name, step_description } => {
                body.push_str(
                    &format!(
                        "\"{}: {}\" has no matching requiremeeeeeeeeeeent.",
                        step_name,
                        step_description
                    )
                )
            },
            RoswaalCompilationErrorCode::NoRequirementStep { requirement_name, requirement_description } => {
                body.push_str(
                    &format!(
                        "\"{}: {}\" has no matching steeeeeeeeeeep.",
                        requirement_name,
                        requirement_description
                    )
                )
            },
            RoswaalCompilationErrorCode::UnknownLocationName(name) => {
                body.push_str(
                    &format!(
                        "\"{}\" is an unknown location naaaaaaaame. Add it using the `/add-locations` commaaaaaand!",
                        name
                    )
                )
            },
            RoswaalCompilationErrorCode::InvalidLocationName(name, error) => {
                match error {
                    // NB: This case is treated as the "NoCommandDescription" error.
                    RoswaalLocationNameParsingError::Empty => {},
                    RoswaalLocationNameParsingError::InvalidFormat => {
                        body.push_str(
                            &format!(
                                "\"{}\" was in an invalid foooooormat. Make sure you don't include any special characters in the location naaaaaaaaaame.",
                                name
                            )
                        )
                    }
                }
            },
            RoswaalCompilationErrorCode::InvalidCommandName(name) => {
                body.push_str(
                    &format!(
                        "\"{}\" has valid command syyyyyntax, but it is not a known comaaaaand.",
                        name
                    )
                )
            },
            RoswaalCompilationErrorCode::Duplicate { name, code } => {
                match code {
                    RoswaalCompilationDuplicateErrorCode::StepLabel => {
                        body.push_str(
                            &format!(
                                "Mutliple steps named \"{}\" were fooooound! Make sure there is only ooooone!",
                                name
                            )
                        )
                    },
                    RoswaalCompilationDuplicateErrorCode::RequirementLabel => {
                        body.push_str(
                            &format!(
                                "Mutliple requirements named \"{}\" were fooooound! Make sure there is only ooooone!",
                                name
                            )
                        )
                    }
                }
            },
            RoswaalCompilationErrorCode::TestNameAlreadyDeclared => {
                body.push_str("This test has multiple \"New Test\" commaaaaaaands. Make sure there is only oooooooone!")
            },
        }
        body.push_str("\n");
        body.push_str(&format!("*Line: {}*", self.error.line_number()));
        SlackSection::from_markdown(&body)
    }
}

#[cfg(test)]
mod tests {
    use crate::{language::ast::RoswaalTestSyntax, operations::add_tests::{AddTestsStatus, RoswaalTestCompilationResults}, slack::ui_lib::test_support::{assert_slack_view_snapshot, SnapshotMode}};

    use super::AddTestsView;

    #[test]
    fn success_no_compile_errors_snapshot() {
        let tests = vec![
            RoswaalTestSyntax::from("\
New Test: Big Chungus
Step 1: Big
Requirement 1: Chungus
"),
            RoswaalTestSyntax::from("\
New Test: Big Chungus II
Step 1: Big
Requirement 1: Chungus
"),
            RoswaalTestSyntax::from("\
New Test: Big Chungus III
Step 1: Big
Requirement 1: Chungus
")
        ];
        let results = RoswaalTestCompilationResults::compile(&tests, &vec![]);
        assert_slack_view_snapshot(
            "add-tests-success-no-compilation-errors",
            &AddTestsView::new(AddTestsStatus::Success { results, should_warn_undeleted_branch: false }),
            SnapshotMode::Comparing
        )
    }

    #[test]
    fn success_all_compile_errors_snapshot() {
        let tests = vec![
            RoswaalTestSyntax::from(""),
            RoswaalTestSyntax::from("New Test: Big Chungus II"),
            RoswaalTestSyntax::from("\
New Test: Big Chungus III
Step 1: Big
"),
            RoswaalTestSyntax::from("\
New Test: Big Chungus III
Requirement 1: Big
"),
            RoswaalTestSyntax::from("\
New Test: Big Chungus III
Step 1: Big
Step 1: Big 2
"),
            RoswaalTestSyntax::from("\
New Test: Big Chungus III
Requirement 1: Big
Requirement 2: Big 2
"),
            RoswaalTestSyntax::from("\
New Test: Big Chungus IIII
Step 1: Big
Requirement 1: Chungus
Set Location: The Middle of Nowhere
"),
            RoswaalTestSyntax::from("\
New Test: Big Chungus IIII
Step 1: Big
Requirement 1: Chungus
Set Location: IN09*09809480valid
"),
            RoswaalTestSyntax::from("\
New Test: Big Chungus IIII
Step 1: Big
Requirement 1: Chungus
New Test: Big Chungus IIIII
"),
            RoswaalTestSyntax::from("\
asioljdoasjodjasodjosa
"),
            RoswaalTestSyntax::from("\
New Test: Hello World
Fake Command: This is a fake command!
")
        ];
        let results = RoswaalTestCompilationResults::compile(&tests, &vec![]);
        assert_slack_view_snapshot(
            "add-tests-success-all-compilation-errors",
            &AddTestsView::new(AddTestsStatus::Success { results, should_warn_undeleted_branch: false }),
            SnapshotMode::Comparing
        )
    }

    #[test]
    fn success_mixed_compilation_results_snapshot() {
        let tests = vec![
            RoswaalTestSyntax::from("\
New Test: Big Chungus III
Step 1: Big
"),
            RoswaalTestSyntax::from("\
New Test: This is a Valid Test
Step 1: Thing
Requirement 1: Thhing
")
        ];
        let results = RoswaalTestCompilationResults::compile(&tests, &vec![]);
        assert_slack_view_snapshot(
            "add-tests-success-mixed-compilation-results",
            &AddTestsView::new(AddTestsStatus::Success { results, should_warn_undeleted_branch: true }),
            SnapshotMode::Comparing
        )
    }

    #[test]
    fn success_warn_undeleted_branch_snapshot() {
        let tests = vec![
            RoswaalTestSyntax::from("\
New Test: Big Chungus
Step 1: Big
Requirement 1: Chungus
")
        ];
        let results = RoswaalTestCompilationResults::compile(&tests, &vec![]);
        assert_slack_view_snapshot(
            "add-tests-success-warn-undeleted-branch-errors",
            &AddTestsView::new(AddTestsStatus::Success { results, should_warn_undeleted_branch: true }),
            SnapshotMode::Comparing
        )
    }

    #[test]
    fn no_tests_found_snapshot() {
        assert_slack_view_snapshot(
            "add-tests-no-tests-found",
            &AddTestsView::new(AddTestsStatus::NoTestsFound),
            SnapshotMode::Comparing
        )
    }

    #[test]
    fn pr_fail_snapshot() {
        assert_slack_view_snapshot(
            "add-tests-pr-fail",
            &AddTestsView::new(AddTestsStatus::FailedToOpenPullRequest),
            SnapshotMode::Comparing
        )
    }

    #[test]
    fn merge_conflict_snapshot() {
        assert_slack_view_snapshot(
            "add-tests-merge-conflict",
            &AddTestsView::new(AddTestsStatus::MergeConflict),
            SnapshotMode::Comparing
        )
    }
}
