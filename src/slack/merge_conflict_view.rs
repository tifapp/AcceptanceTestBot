use super::ui_lib::{block_kit_views::SlackSection, slack_view::SlackView};

/// A view for representing a merge conflict from an operation.
pub struct MergeConflictView {
    slack_user_id: String,
}

impl MergeConflictView {
    pub fn new(slack_user_id: &str) -> Self {
        Self {
            slack_user_id: slack_user_id.to_string(),
        }
    }
}

impl SlackView for MergeConflictView {
    fn slack_body(&self) -> impl SlackView {
        SlackSection::from_markdown("🔴 *CRITICAL: MERGE CONFLICT DETECTED*")
            .flat_chain_block(
                SlackSection::from_markdown(
                    &format!(
                        "_Fixing this requires manual interveeeeeeention, which requires <@{}> to be useful for ooooonce!_",
                        self.slack_user_id
                    )
                )
            )
    }
}

#[cfg(test)]
mod tests {
    use crate::slack::{
        ui_lib::test_support::{assert_slack_view_snapshot, SnapshotMode},
        users::MATTHEW_SLACK_USER_ID,
    };

    use super::MergeConflictView;

    #[test]
    fn snapshot() {
        assert_slack_view_snapshot(
            "merge-conflict",
            &MergeConflictView::new(MATTHEW_SLACK_USER_ID),
            SnapshotMode::Comparing,
        )
    }
}
