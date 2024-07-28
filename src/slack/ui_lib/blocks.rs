use serde::Serialize;

use super::{primitive_view::PrimitiveView, slack_view::SlackView};

/// A struct containing a flat array of slack blocks.
///
/// You create instances of this struct via the `render_slack_blocks` function which will convert
/// a `SlackView` hierarchy into a flat array of JSON-serializeable slack blocks.
#[derive(Debug, PartialEq, Eq, Serialize, Clone)]
pub struct SlackBlocks(_SlackBlocksCollection);

impl SlackBlocks {
    pub(super) fn from(blocks: _SlackBlocksCollection) -> Self {
        Self(blocks)
    }
}

impl SlackBlocks {
    pub(super) fn collection(&self) -> &_SlackBlocksCollection {
        &self.0
    }
}

/// A collection of slack blocks.
///
/// This struct is an implementation detail of the library, and it could be removed or changed in
/// the future. Do not depend on this struct directly.
#[derive(Debug, PartialEq, Eq, Serialize, Clone)]
pub struct _SlackBlocksCollection(Vec<serde_json::Value>);

impl _SlackBlocksCollection {
    pub(super) fn new() -> Self {
        Self(vec![])
    }
}

impl _SlackBlocksCollection {
    pub(super) fn push_view(&mut self, view: &impl SlackView) {
        view.slack_body().__push_blocks_into(self)
    }

    pub(super) fn push_primitive_view(&mut self, view: &PrimitiveView) {
        if let Some(value) = view.json_value() {
            self.0.push(value.clone())
        }
    }

    pub(super) fn extend(&mut self, other: &Self) {
        self.0.extend(other.0.iter().map(|v| v.to_owned()))
    }
}
