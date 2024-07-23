use serde::Serialize;
use serde_json::json;

use super::{blocks::_SlackBlocks, slack_view::SlackView};

#[derive(Debug, Clone)]
pub(super) struct PrimitiveView {
    json: Option<serde_json::Value>
}

impl PrimitiveView {
    pub(super) fn new(view: &(impl SlackView + Serialize)) -> Self {
        Self { json: Some(json!(view)) }
    }

    pub(super) fn empty() -> Self {
        Self { json: None }
    }
}

impl PrimitiveView {
    pub(super) fn json_value(&self) -> Option<&serde_json::Value> {
        self.json.as_ref()
    }
}

impl SlackView for PrimitiveView {
    fn _push_blocks_into(&self, slack_blocks: &mut _SlackBlocks) {
        slack_blocks.push_primitive_view(self)
    }

    #[allow(refining_impl_trait)]
    fn slack_body(&self) -> PrimitiveView {
        panic!("Do not directly call slack_body on views, as a view may not have a body due to it being a primitive view.")
    }
}
