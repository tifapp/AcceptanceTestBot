use super::ui_lib::{block_kit_views::SlackSection, slack_view::SlackView};

/// A view for warning the branch created by an operation was unable to be deleted.
pub struct WarnUndeletedBranchView;

impl SlackView for WarnUndeletedBranchView {
    fn slack_body(&self) -> impl SlackView {
        SlackSection::from_markdown(
            "🟡 _The local branch created by this operation was not deleted._",
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::slack::ui_lib::test_support::{assert_slack_view_snapshot, SnapshotMode};

    use super::WarnUndeletedBranchView;

    #[test]
    fn snapshot() {
        assert_slack_view_snapshot(
            "warn-undeleted-branch",
            &WarnUndeletedBranchView,
            SnapshotMode::Comparing,
        )
    }
}
