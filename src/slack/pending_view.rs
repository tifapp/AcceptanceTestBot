use super::ui_lib::{block_kit_views::SlackSection, slack_view::SlackView};

pub struct PendingView;

impl SlackView for PendingView {
    fn slack_body(&self) -> impl SlackView {
        SlackSection::from_markdown("⏳ I am ooooooooooon that!")
    }
}

#[cfg(test)]
mod tests {
    use crate::slack::ui_lib::test_support::{assert_slack_view_snapshot, SnapshotMode};

    use super::PendingView;

    #[test]
    fn snapshot() {
        assert_slack_view_snapshot("pending", &PendingView, SnapshotMode::Comparing)
    }
}
