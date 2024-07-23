use serde::Serialize;

use super::{primitive_view::PrimitiveView, slack_view::SlackView};

/// A struct to render a `SlackView` into a flat array of serialized slack blocks.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct SlackBlocks(Vec<serde_json::Value>);

impl SlackBlocks {
    /// Renders the specified view into a `SlackBlocks` instance.
    pub fn render(view: &impl SlackView) -> Self {
        let mut blocks = _SlackBlocks::new();
        view._push_blocks_into(&mut blocks);
        Self(blocks.0)
    }
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct _SlackBlocks(Vec<serde_json::Value>);

impl _SlackBlocks {
    pub(super) fn new() -> Self {
        Self(vec![])
    }
}

impl _SlackBlocks {
    pub(super) fn push_view(&mut self, view: &impl SlackView) {
        view.slack_body()._push_blocks_into(self)
    }

    pub(super) fn push_primitive_view(&mut self, view: &PrimitiveView) {
        if let Some(value) = view.json_value() {
            self.0.push(value.clone())
        }
    }

    pub(super) fn extend(&mut self, other: &SlackBlocks) {
        self.0.extend(other.0.iter().map(|v| v.to_owned()))
    }
}
