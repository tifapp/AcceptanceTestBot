use std::backtrace::Backtrace;

use anyhow::Error;

use super::ui_lib::{block_kit_views::{SlackHeader, SlackSection}, slack_view::SlackView};

///
pub struct ErrorView {
    error: Error
}

impl ErrorView {
    pub fn new(error: Error) -> Self {
        Self { error }
    }
}

impl SlackView for ErrorView {
    fn slack_body(&self) -> impl SlackView {
        SlackHeader::new("An Error Occurred")
            .flat_chain_block(SlackSection::from_markdown("🔴 *Error*"))
            .flat_chain_block(
                SlackSection::from_plaintext(&self.error.to_string())
                    .emoji_enabled(false)
            )
            .flat_chain_block(SlackSection::from_markdown("⚠️ *Stack Trace*"))
            .flat_chain_block(
                SlackSection::from_plaintext(&self.error.backtrace().to_string())
                    .emoji_enabled(false)
            )
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use anyhow::Error;
    use dotenv::dotenv;

    use crate::{slack::ui_lib::test_support::{assert_slack_view_snapshot, SnapshotMode}, utils::test_error::TestError};

    use super::ErrorView;

    #[test]
    fn snapshot() {
        dotenv().unwrap();
        env::var("RUST_BACKTRACE")
            .expect("Set RUST_BACKTRACE=1 in the .env to view the ErrorView slack snapshot.");
        assert_slack_view_snapshot(
            "error-view",
            &ErrorView::new(Error::new(TestError)),
            SnapshotMode::Comparing
        )
    }
}
