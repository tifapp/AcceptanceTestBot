use super::ui_lib::{block_kit_views::SlackSection, slack_view::SlackView};

/// A view for indicating that a pull request couldn't be opened.
pub struct FailedToOpenPullRequestView;

impl SlackView for FailedToOpenPullRequestView {
    fn slack_body(&self) -> impl SlackView {
        SlackSection::from_markdown("🔴 *Error: Failed to open Pull Request*")
            .flat_chain_block(
                SlackSection::from_markdown("_The pull request could not be opeeeeened. Check the logs for deeeeeeetails._")
            )
    }
}

#[cfg(test)]
mod tests {
    use crate::slack::ui_lib::test_support::assert_slack_view_snapshot;

    use super::FailedToOpenPullRequestView;

    #[test]
    fn snapshot() {
        assert_slack_view_snapshot(
            "failed-to-open-pull-request",
            &FailedToOpenPullRequestView,
            false
        )
    }
}
