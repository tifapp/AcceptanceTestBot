use std::backtrace::Backtrace;

use super::ui_lib::{block_kit_views::{SlackHeader, SlackSection}, slack_view::SlackView};

pub struct ErrorView {
    backtrace: Backtrace
}

impl ErrorView {
    pub fn new(backtrace: Backtrace) -> Self {
        Self { backtrace }
    }
}

impl SlackView for ErrorView {
    fn slack_body(&self) -> impl SlackView {
        SlackHeader::new("An Error Occurred")
            .flat_chain_block(SlackSection::from_plaintext(&self.backtrace.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use std::{backtrace::Backtrace, env};

    use dotenv::dotenv;

    use crate::slack::ui_lib::test_support::{assert_slack_view_snapshot, SnapshotMode};

    use super::ErrorView;

    #[test]
    fn snapshot() {
        dotenv().unwrap();
        env::var("RUST_BACKTRACE")
            .expect("Set RUST_BACKTRACE=1 in the .env to view the ErrorView slack snapshot.");
        let backtrace = Backtrace::capture();
        assert_slack_view_snapshot(
            "error-view",
            &ErrorView::new(backtrace),
            SnapshotMode::Recording
        )
    }
}
