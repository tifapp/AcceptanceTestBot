use serde::Serialize;

use super::{blocks::SlackBlocks, slack_view::SlackView};

/// A slack message.
///
/// A slack message is created from a `SlackView` and a string channel identifier.
#[derive(Debug, Serialize)]
pub struct SlackMessage {
    #[serde(rename = "channel")]
    channel_id: String,
    blocks: SlackBlocks
}

impl SlackMessage {
    /// Constructs a slack message for testing.
    pub fn for_testing(view: &impl SlackView) -> Self {
        Self::new("__TEST__", view)
    }

    /// Constructs a slack message for the tif acceptance tests channel.
    pub fn for_tif_acceptance_tests(view: &impl SlackView) -> Self {
        Self::new("C06PSMAB7QV", view)
    }

    /// Constructs a slack message from a channel id and `SlackView`.
    pub fn new(channel_id: &str, view: &impl SlackView) -> Self {
        Self { channel_id: channel_id.to_string(), blocks: SlackBlocks::render(view) }
    }
}
