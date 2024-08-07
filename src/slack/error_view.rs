use std::{backtrace::Backtrace, cmp::min};

use anyhow::Error;

use super::ui_lib::{
    block_kit_views::{SlackDivider, SlackHeader, SlackSection},
    if_view::If,
    slack_view::SlackView,
};

pub struct ErrorView {
    error: Error,
}

impl ErrorView {
    pub fn new(error: Error) -> Self {
        Self { error }
    }
}

const SECTION_MAX_CHARACTERS: usize = 3000;

impl SlackView for ErrorView {
    fn slack_body(&self) -> impl SlackView {
        let backtrace_string = self.error.backtrace().to_string();
        let len = min(backtrace_string.len(), SECTION_MAX_CHARACTERS);
        SlackHeader::new("An Error Occurred")
            .flat_chain_block(SlackSection::from_markdown("🔴 *Error*"))
            .flat_chain_block(
                SlackSection::from_plaintext(&self.error.to_string()).emoji_enabled(false),
            )
            .flat_chain_block(SlackSection::from_markdown("⚠️ *Stack Trace*"))
            .flat_chain_block(
                SlackSection::from_plaintext(&backtrace_string[0..len]).emoji_enabled(false),
            )
            .flat_chain_block(If::is_true(
                backtrace_string.len() > SECTION_MAX_CHARACTERS,
                || {
                    SlackDivider.flat_chain_block(SlackSection::from_markdown(
                        "🟡 _Stack Trace truncated due to 3000 character limit..._",
                    ))
                },
            ))
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use anyhow::Error;
    use dotenv::dotenv;

    use crate::{
        slack::ui_lib::test_support::{assert_slack_view_snapshot, SnapshotMode},
        utils::test_error::TestError,
    };

    use super::ErrorView;

    #[test]
    fn snapshot() {
        dotenv().unwrap();
        env::var("RUST_BACKTRACE")
            .expect("Set RUST_BACKTRACE=1 in the .env to view the ErrorView slack snapshot.");
        assert_slack_view_snapshot(
            "error-view",
            &ErrorView::new(Error::new(TestError)),
            SnapshotMode::Comparing,
        )
    }
}
