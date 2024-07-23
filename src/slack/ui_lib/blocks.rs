use serde::Serialize;

use super::{primitive_view::_PrimitiveView, slack_view::SlackView};

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

    pub(super) fn push_primitive_view(&mut self, view: &_PrimitiveView) {
        if let Some(value) = view.json_value() {
            self.0.push(value.clone())
        }
    }
}
