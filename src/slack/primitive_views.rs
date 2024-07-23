use std::fmt::Debug;

use serde::{ser::SerializeSeq, Serialize, Serializer};
use serde_json::json;

use super::{blocks::_SlackBlocks, slack_view::SlackView};

/// A section component.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct SlackSection {
    #[serde(rename = "type")]
    _type: &'static str,
    text: SlackMarkdownText
}

impl SlackSection {
    /// A convenience constructor to create a section from markdown.
    pub fn from_markdown(markdown: &str) -> Self {
        Self {
            _type: "section",
            text: SlackMarkdownText::new(markdown)
        }
    }
}

impl SlackView for SlackSection {
    fn slack_body(&self) -> impl SlackView { PrimitiveView::new(self) }
}

/// Slack Markdown Text for use in a Section.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct SlackMarkdownText {
    #[serde(rename = "type")]
    _type: &'static str,
    text: String
}

impl SlackMarkdownText {
    pub fn new(markdown: &str) -> Self {
        Self { _type: "mrkdwn", text: markdown.to_string() }
    }
}

impl SlackView for SlackMarkdownText {
    fn slack_body(&self) -> impl SlackView { PrimitiveView::new(self) }
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct _SlackDivider {
    #[serde(rename = "type")]
    _type: &'static str
}

/// A slack divider component.
#[allow(nonstandard_style)]
pub const SlackDivider: _SlackDivider = _SlackDivider { _type: "divider" };

impl SlackView for _SlackDivider {
    fn slack_body(&self) -> impl SlackView { PrimitiveView::new(self) }
}

#[derive(Debug, PartialEq, Eq)]
pub struct _FlatChainSlackView<Base: SlackView, Other: SlackView> {
    base: Base,
    other: Other
}

impl <Base: SlackView + 'static, Other: SlackView + 'static>
    _FlatChainSlackView<Base, Other> {
    pub(super) fn new(base: Base, other: Other) -> Self {
        Self { base, other }
    }
}

impl <Base: SlackView + 'static, Other: SlackView + 'static> SlackView
    for _FlatChainSlackView<Base, Other> {
    fn slack_body(&self) -> impl SlackView { PrimitiveView::empty() }

    fn _push_blocks_into(&self, slack_blocks: &mut _SlackBlocks) {
        self.base._push_blocks_into(slack_blocks);
        self.other._push_blocks_into(slack_blocks)
    }
}

#[derive(Debug)]
pub(super) struct PrimitiveView {
    json: Option<serde_json::Value>
}

impl PrimitiveView {
    fn new(view: &(impl SlackView + Serialize)) -> Self {
        Self { json: Some(json!(view)) }
    }

    fn empty() -> Self {
        Self { json: None }
    }
}

impl PrimitiveView {
    pub(super) fn value(&self) -> Option<&serde_json::Value> {
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

#[cfg(test)]
mod tests {
    use crate::slack::test_support::assert_blocks_json;

    use super::*;

    #[derive(Debug, Serialize)]
    struct DividersView;

    impl SlackView for DividersView {
        fn slack_body(&self) -> impl SlackView {
            SlackDivider.flat_chain_block(SlackDivider)
                .flat_chain_block(
                    SlackDivider.flat_chain_block(
                        SlackDivider.flat_chain_block(SlackDivider)
                    )
                )
                .flat_chain_block(SlackDivider)
        }
    }

    #[test]
    fn flat_chain_flattens_nested_dividers() {
        assert_blocks_json(
            &DividersView,
            r#"[{"type":"divider"},{"type":"divider"},{"type":"divider"},{"type":"divider"},{"type":"divider"},{"type":"divider"}]"#
        );
    }

    #[derive(Debug, Serialize)]
    struct TextView;

    impl SlackView for TextView {
        fn slack_body(&self) -> impl SlackView {
            SlackSection::from_markdown("Hello World!")
        }
    }

    #[derive(Debug, Serialize)]
    struct NestedView;

    impl SlackView for NestedView {
        fn slack_body(&self) -> impl SlackView {
            TextView.flat_chain_block(TextView)
        }
    }

    #[test]
    fn nested_view_flattens_to_proper_json() {
        assert_blocks_json(
            &NestedView,
            r#"[{"text":{"text":"Hello World!","type":"mrkdwn"},"type":"section"},{"text":{"text":"Hello World!","type":"mrkdwn"},"type":"section"}]"#
        );
    }
}
