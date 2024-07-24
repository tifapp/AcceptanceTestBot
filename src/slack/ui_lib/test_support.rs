use serde::{Deserialize, Serialize};

use super::slack_view::SlackView;
use super::message::SlackMessage;
use std::path::Path;
use std::{fs::File, io::Write};
use std::io::Read;
use super::blocks::SlackBlocks;
use super::slack_view::render_slack_view;

/// Asserts the json rendered by a slack view.
#[cfg(test)]
pub fn assert_blocks_json(view: &impl SlackView, json: &str) {
    let message = SlackMessage::for_testing(view);
    let expected_json = format!("{{\"channel\":\"__TEST__\",\"blocks\":{}}}", json);
    let json = serde_json::to_string(&message).unwrap();
    assert_eq!(json, expected_json)
}

/// An enum to determine the behavior of `assert_slack_view_snapshot`.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum SnapshotMode {
    /// Specifies that a new snapshot should overwrite the existing one without comparing against it.
    Recording,
    /// Specified that a new snapshot should be compared with the existing one.
    ///
    /// If no snapshot exists, then this option is treated the same as `Recording`.
    Comparing
}

/// Asserts a snapshot of a `SlackView`.
///
/// This function is useful for testing and iterating on the UI of complex `SlackView`s. This
/// assertion writes the output of the view to the `slack-snapshots` directory, and each snapshot
/// can be copied and pasted directly into the
/// [Block Kit Builder UI](https://app.slack.com/block-kit-builder).
///
/// `is_recording` is a parameter to update a snapshot when it has changed. When the value is true,
/// the current snapshot will be overidden by the new snapshot, and the test will pass. When the
/// value is false, the new snapshot will be asserted against the old snapshot. The new
/// snapshot will not be written to `slack-snapshots`, but rather the gitignored
/// `slack-snapshots-diffs` directory. This directory is useful for comparing snapshots when a test
/// failure occurs.
#[cfg(test)]
pub fn assert_slack_view_snapshot(
    name: &str,
    view: &impl SlackView,
    mode: SnapshotMode
) {
    let raw_path = format!("./slack-snapshots/{}.json", name);
    let path = Path::new(&raw_path);
    let is_recording = mode == SnapshotMode::Recording || !path.exists();
    let blocks = BlockKitBuilderCompatibleBlocks { blocks: render_slack_view(view) };
    let blocks_json = serde_json::to_string(&blocks).unwrap();
    if is_recording {
        let mut file = File::create(path).unwrap();
        _ = file.write(blocks_json.as_bytes());
    } else {
        let diff_path = format!("./slack-snapshot-diffs/{}.json", name);
        let mut file = File::create(&diff_path).unwrap();
        _ = file.write(blocks_json.as_bytes());
        file = File::open(path).unwrap();
        let mut snapshot_json = String::new();
        _ = file.read_to_string(&mut snapshot_json);
        assert_eq!(blocks_json, snapshot_json)
    }
}

#[derive(Debug, Serialize)]
struct BlockKitBuilderCompatibleBlocks {
    blocks: SlackBlocks
}

#[cfg(test)]
mod tests {
    use crate::slack::ui_lib::{block_kit_views::SlackSection, slack_view::SlackView};

    use super::{assert_slack_view_snapshot, SnapshotMode};

    struct SomeView;

    impl SlackView for SomeView {
        fn slack_body(&self) -> impl SlackView {
            SlackSection::from_markdown("I am a test view")
        }
    }

    #[test]
    fn record_snapshot() {
        assert_slack_view_snapshot("test-snapshot", &SomeView, SnapshotMode::Comparing)
    }
}
