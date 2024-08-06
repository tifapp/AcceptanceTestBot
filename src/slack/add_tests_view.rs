use std::borrow::Borrow;

use crate::{
    language::{
        ast::RoswaalTestSyntax,
        compilation_results::RoswaalTestCompilationFailure,
        compiler::{
            RoswaalCompilationDuplicateErrorCode, RoswaalCompilationError,
            RoswaalCompilationErrorCode,
        },
        test::RoswaalCompiledTest,
    },
    location::name::RoswaalLocationNameParsingError,
    operations::add_tests::AddTestsStatus,
};

use super::{
    merge_conflict_view::MergeConflictView,
    pr_open_fail_view::FailedToOpenPullRequestView,
    ui_lib::{
        block_kit_views::{SlackDivider, SlackHeader, SlackSection},
        for_each_view::ForEachView,
        if_view::If,
        slack_view::SlackView,
    },
    users::MATTHEW_SLACK_USER_ID,
    warn_undeleted_branch_view::WarnUndeletedBranchView,
};

pub struct AddTestsView<'r> {
    status: AddTestsStatus<'r>,
}

impl<'r> AddTestsView<'r> {
    pub fn new(status: AddTestsStatus<'r>) -> Self {
        Self { status }
    }
}

impl<'r> SlackView for AddTestsView<'r> {
    fn slack_body(&self) -> impl SlackView {
        SlackHeader::new("Add Tests").flat_chain_block(self.status_view())
    }
}

impl<'r> AddTestsView<'r> {
    fn status_view(&self) -> impl SlackView {
        match self.status.borrow() {
            AddTestsStatus::Success {
                results,
                should_warn_undeleted_branch,
            } => {
                If::is_true(
                    results.has_compiling_tests(),
                    || self.compiling_tests_view(&results.tests_with_syntax())
                )
                .flat_chain_block(
                    If::is_true(
                        results.has_non_compiling_tests(),
                        || self.non_compiling_tests_view(&results.failures())
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
            }
            AddTestsStatus::NoTestsFound => {
                SlackSection::from_markdown("🔴 No tests were fooooooound.").erase_to_any_view()
            }
            AddTestsStatus::MergeConflict => {
                MergeConflictView::new(MATTHEW_SLACK_USER_ID).erase_to_any_view()
            }
            AddTestsStatus::FailedToOpenPullRequest => {
                FailedToOpenPullRequestView.erase_to_any_view()
            }
        }
    }
}

impl<'r> AddTestsView<'r> {
    fn compiling_tests_view(
        &self,
        tests_with_syntax: &Vec<(RoswaalCompiledTest, RoswaalTestSyntax<'r>)>,
    ) -> impl SlackView {
        let iter = tests_with_syntax
            .iter()
            .map(|e| e.clone())
            .enumerate()
            .map(|(index, value)| (index < tests_with_syntax.len() - 1, value));
        SlackSection::from_markdown("✅ *The following tests were compiled succeeeeeeeeessfully!*")
            .flat_chain_block(ForEachView::new(
                iter,
                |(is_showing_divider, (test, syntax))| {
                    SlackSection::from_markdown(&format!("📝 *{}*", test.name()))
                        .flat_chain_block(SlackSection::from_markdown(
                            &syntax.markdown_code_block(),
                        ))
                        .flat_chain_block(If::is_true(*is_showing_divider, || SlackDivider))
                },
            ))
            .erase_to_any_view()
    }

    fn non_compiling_tests_view(
        &self,
        failures: &Vec<RoswaalTestCompilationFailure<'r>>,
    ) -> impl SlackView {
        let iter = failures
            .iter()
            .map(|e| e.clone())
            .enumerate()
            .map(|(index, value)| (index < failures.len() - 1, value));
        SlackSection::from_markdown(
            "⚠️ *The following tests did not compile succeeeeeessfully. They are only listed by the number based on the compilation oooooooooorder!*"
        )
        .flat_chain_block(
            ForEachView::new(iter, |(is_showing_divider, failure)| {
                NonCompilingTestView {
                    test_number: failure.test_number(),
                    errors: failure.errors().to_vec(),
                    syntax_markdown: failure.syntax().markdown_code_block()
                }
                .flat_chain_block(If::is_true(*is_showing_divider, || SlackDivider))
            })
        )
        .erase_to_any_view()
    }
}

struct NonCompilingTestView {
    test_number: usize,
    errors: Vec<RoswaalCompilationError>,
    syntax_markdown: String,
}

impl SlackView for NonCompilingTestView {
    fn slack_body(&self) -> impl SlackView {
        SlackSection::from_markdown(&format!("❗️ *Test {}*", self.test_number))
            .flat_chain_block(SlackSection::from_markdown(&self.syntax_markdown))
            .flat_chain_block(ForEachView::new(self.errors.iter(), |error| {
                CompilationErrorView { error }
            }))
    }
}

struct CompilationErrorView<'v> {
    error: &'v RoswaalCompilationError,
}

impl<'v> SlackView for CompilationErrorView<'v> {
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
    use crate::{
        language::{ast::RoswaalTestSyntax, compilation_results::RoswaalTestCompilationResults},
        operations::add_tests::AddTestsStatus,
        slack::ui_lib::test_support::{assert_slack_view_snapshot, SnapshotMode},
    };

    use super::AddTestsView;

    #[test]
    fn success_no_compile_errors_snapshot() {
        let tests = vec![
            RoswaalTestSyntax::from(
                "\
New Test: Big Chungus
Step 1: Big
Requirement 1: Chungus
",
            ),
            RoswaalTestSyntax::from(
                "\
New Test: Big Chungus II
Step 1: Big
Requirement 1: Chungus
",
            ),
            RoswaalTestSyntax::from(
                "\
New Test: Big Chungus III
Step 1: Big
Requirement 1: Chungus
",
            ),
        ];
        let results = RoswaalTestCompilationResults::compile(&tests, &vec![]);
        assert_slack_view_snapshot(
            "add-tests-success-no-compilation-errors",
            &AddTestsView::new(AddTestsStatus::Success {
                results,
                should_warn_undeleted_branch: false,
            }),
            SnapshotMode::Comparing,
        )
    }

    #[test]
    fn success_all_compile_errors_snapshot() {
        let tests = vec![
            RoswaalTestSyntax::from(""),
            RoswaalTestSyntax::from("New Test: Big Chungus II"),
            RoswaalTestSyntax::from(
                "\
New Test: Big Chungus III
Step 1: Big
",
            ),
            RoswaalTestSyntax::from(
                "\
New Test: Big Chungus III
Requirement 1: Big
",
            ),
            RoswaalTestSyntax::from(
                "\
New Test: Big Chungus III
Step 1: Big
Step 1: Big 2
",
            ),
            RoswaalTestSyntax::from(
                "\
New Test: Big Chungus III
Requirement 1: Big
Requirement 2: Big 2
",
            ),
            RoswaalTestSyntax::from(
                "\
New Test: Big Chungus IIII
Step 1: Big
Requirement 1: Chungus
Set Location: The Middle of Nowhere
",
            ),
            RoswaalTestSyntax::from(
                "\
New Test: Big Chungus IIII
Step 1: Big
Requirement 1: Chungus
Set Location: IN09*09809480valid
",
            ),
            RoswaalTestSyntax::from(
                "\
New Test: Big Chungus IIII
Step 1: Big
Requirement 1: Chungus
New Test: Big Chungus IIIII
",
            ),
            RoswaalTestSyntax::from(
                "\
asioljdoasjodjasodjosa
",
            ),
            RoswaalTestSyntax::from(
                "\
New Test: Hello World
Fake Command: This is a fake command!
",
            ),
        ];
        let results = RoswaalTestCompilationResults::compile(&tests, &vec![]);
        assert_slack_view_snapshot(
            "add-tests-success-all-compilation-errors",
            &AddTestsView::new(AddTestsStatus::Success {
                results,
                should_warn_undeleted_branch: false,
            }),
            SnapshotMode::Comparing,
        )
    }

    #[test]
    fn success_mixed_compilation_results_snapshot() {
        let tests = vec![
            RoswaalTestSyntax::from(
                "\
New Test: Big Chungus III
Step 1: Big
",
            ),
            RoswaalTestSyntax::from(
                "\
New Test: This is a Valid Test
Step 1: Thing
Requirement 1: Thhing
",
            ),
        ];
        let results = RoswaalTestCompilationResults::compile(&tests, &vec![]);
        assert_slack_view_snapshot(
            "add-tests-success-mixed-compilation-results",
            &AddTestsView::new(AddTestsStatus::Success {
                results,
                should_warn_undeleted_branch: true,
            }),
            SnapshotMode::Comparing,
        )
    }

    #[test]
    fn success_warn_undeleted_branch_snapshot() {
        let tests = vec![RoswaalTestSyntax::from(
            "\
New Test: Big Chungus
Step 1: Big
Requirement 1: Chungus
",
        )];
        let results = RoswaalTestCompilationResults::compile(&tests, &vec![]);
        assert_slack_view_snapshot(
            "add-tests-success-warn-undeleted-branch-errors",
            &AddTestsView::new(AddTestsStatus::Success {
                results,
                should_warn_undeleted_branch: true,
            }),
            SnapshotMode::Comparing,
        )
    }

    #[test]
    fn no_tests_found_snapshot() {
        assert_slack_view_snapshot(
            "add-tests-no-tests-found",
            &AddTestsView::new(AddTestsStatus::NoTestsFound),
            SnapshotMode::Comparing,
        )
    }

    #[test]
    fn pr_fail_snapshot() {
        assert_slack_view_snapshot(
            "add-tests-pr-fail",
            &AddTestsView::new(AddTestsStatus::FailedToOpenPullRequest),
            SnapshotMode::Comparing,
        )
    }

    #[test]
    fn merge_conflict_snapshot() {
        assert_slack_view_snapshot(
            "add-tests-merge-conflict",
            &AddTestsView::new(AddTestsStatus::MergeConflict),
            SnapshotMode::Comparing,
        )
    }
}
