use serde::Serialize;
use serde_json::json;

use super::{blocks::_SlackBlocks, slack_view::SlackView};

#[derive(Debug, Clone)]
pub struct _PrimitiveView {
    json: Option<serde_json::Value>
}

impl _PrimitiveView {
    pub(super) fn new(view: &(impl SlackView + Serialize)) -> Self {
        Self { json: Some(json!(view)) }
    }

    pub(super) fn empty() -> Self {
        Self { json: None }
    }
}

impl _PrimitiveView {
    pub(super) fn json_value(&self) -> Option<&serde_json::Value> {
        self.json.as_ref()
    }
}

impl SlackView for _PrimitiveView {
    fn _push_blocks_into(&self, slack_blocks: &mut _SlackBlocks) {
        slack_blocks.push_primitive_view(self)
    }

    fn _as_primitive_view(&self) -> Option<_PrimitiveView> {
        Some(self.clone())
    }

    #[allow(refining_impl_trait)]
    fn slack_body(&self) -> _PrimitiveView {
        panic!("Do not directly call slack_body on views, as a view may not have a body due to it being a primitive view.")
    }
}
