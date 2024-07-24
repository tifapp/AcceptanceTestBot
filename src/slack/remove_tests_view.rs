use std::borrow::Borrow;

use crate::operations::remove_tests::RemoveTestsStatus;

use super::{merge_conflict_view::MergeConflictView, pr_open_fail_view::FailedToOpenPullRequestView, ui_lib::{block_kit_views::{SlackDivider, SlackHeader, SlackSection}, if_view::If, slack_view::SlackView}, users::MATTHEW_SLACK_USER_ID, warn_undeleted_branch_view::WarnUndeletedBranchView};

pub struct RemoveTestsView {
    status: RemoveTestsStatus
}

impl RemoveTestsView {
    pub fn new(status: RemoveTestsStatus) -> Self {
        Self { status }
    }
}

impl SlackView for RemoveTestsView {
    fn slack_body(&self) -> impl SlackView {
        SlackHeader::new("Remove Tests")
            .flat_chain_block(self.status_view())
    }
}

impl RemoveTestsView {
    fn status_view(&self) -> impl SlackView {
        match self.status.borrow() {
            RemoveTestsStatus::Success { removed_test_names, should_warn_undeleted_branch } => {
                self.test_names_view(removed_test_names)
                    .flat_chain_block(SlackHeader::new("Next Steps"))
                    .flat_chain_block(
                        SlackSection::from_markdown("Approve the PR found in <#C01B7FFKDCP> to finish the remooooooval!")
                    )
                    .flat_chain_block(
                        If::is_true(
                            *should_warn_undeleted_branch,
                            || SlackDivider.flat_chain_block(WarnUndeletedBranchView)
                        )
                    )
                    .erase_to_any_view()
            },
            RemoveTestsStatus::NoTestsRemoved => {
                SlackSection::from_markdown("🔴 No tests were staged for remoooooooval!")
                    .erase_to_any_view()
            },
            RemoveTestsStatus::FailedToOpenPullRequest => {
                FailedToOpenPullRequestView.erase_to_any_view()
            },
            RemoveTestsStatus::MergeConflict => {
                MergeConflictView::new(MATTHEW_SLACK_USER_ID).erase_to_any_view()
            },
        }
    }

    fn test_names_view(&self, names: &Vec<String>) -> impl SlackView {
        let mut body = "🗑️ *The following tests were staged for remoooooooval!*\n".to_string();
        for name in names {
            body.push_str(&format!("- {}\n", name))
        }
        SlackSection::from_markdown(&body)
    }
}

#[cfg(test)]
mod tests {
    use crate::{operations::remove_tests::RemoveTestsStatus, slack::ui_lib::test_support::assert_slack_view_snapshot};

    use super::RemoveTestsView;

    #[test]
    fn success_snapshot() {
        let removed_test_names = vec![
            "People Die When they are Killed".to_string(),
            "Zanza the Divine".to_string(),
            "L".to_string()
        ];
        assert_slack_view_snapshot(
            "remove-tests-success",
            &RemoveTestsView::new(
                RemoveTestsStatus::Success {
                    removed_test_names,
                    should_warn_undeleted_branch: false
                }
            ),
            false
        )
    }

    #[test]
    fn success_warn_undeleted_branch_snapshot() {
        let removed_test_names = vec![
            "People Die When they are Killed".to_string(),
            "Zanza the Divine".to_string(),
            "L".to_string()
        ];
        assert_slack_view_snapshot(
            "remove-tests-success-warn-undeleted-branch",
            &RemoveTestsView::new(
                RemoveTestsStatus::Success {
                    removed_test_names,
                    should_warn_undeleted_branch: true
                }
            ),
            false
        )
    }

    #[test]
    fn none_removed_snapshot() {
        assert_slack_view_snapshot(
            "remove-tests-none-removed",
            &RemoveTestsView::new(RemoveTestsStatus::NoTestsRemoved),
            false
        )
    }

    #[test]
    fn merge_conflict_snapshot() {
        assert_slack_view_snapshot(
            "remove-tests-merge-conflict",
            &RemoveTestsView::new(RemoveTestsStatus::MergeConflict),
            false
        )
    }

    #[test]
    fn pr_open_failed_snapshot() {
        assert_slack_view_snapshot(
            "remove-tests-pr-fail",
            &RemoveTestsView::new(RemoveTestsStatus::FailedToOpenPullRequest),
            false
        )
    }
}
