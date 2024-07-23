use std::fmt::Debug;

use serde::{ser::SerializeSeq, Serialize, Serializer};
use serde_json::json;

use super::slack_view::SlackView;

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
    fn slack_body(&self) -> impl SlackView { _PrimitiveView }
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
    fn slack_body(&self) -> impl SlackView { _PrimitiveView }
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
    fn slack_body(&self) -> impl SlackView { _PrimitiveView }
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

impl <Base: SlackView + 'static, Other: SlackView + 'static> Serialize
    for _FlatChainSlackView<Base, Other> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut views = self.base._flat_deep_subviews();
        views.append(&mut self.other._flat_deep_subviews());
        let mut seq = serializer.serialize_seq(Some(views.len()))?;
        for view in views {
            seq.serialize_element(&view)?;
        }
        seq.end()
    }
}

impl <Base: SlackView + 'static, Other: SlackView + 'static> SlackView
    for _FlatChainSlackView<Base, Other> {
    fn slack_body(&self) -> impl SlackView { _PrimitiveView }

    fn _flat_deep_subviews(&self) -> Vec<AnySlackView> {
        let mut children = Vec::<AnySlackView>::new();
        children.append(&mut self.base._flat_deep_subviews());
        children.append(&mut self.other._flat_deep_subviews());
        children
    }
}

/// A type-erased Slack View.
#[derive(Debug)]
pub struct AnySlackView {
    json: serde_json::Value
}

impl AnySlackView {
    pub fn from(view: &impl SlackView) -> Self {
        Self { json: json!(view) }
    }
}

impl Serialize for AnySlackView {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        self.json.serialize(serializer)
    }
}

impl SlackView for AnySlackView {
    fn slack_body(&self) -> impl SlackView { _PrimitiveView }
}

#[derive(Debug)]
struct _PrimitiveView;

impl Serialize for _PrimitiveView {
    fn serialize<S>(&self, _: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        panic!("A primitive view cannot be serialized. Avoid directly calling the slack_body method on views, as it may return a primitive view.")
    }
}

impl SlackView for _PrimitiveView {
    #[allow(refining_impl_trait)]
    fn slack_body(&self) -> _PrimitiveView {
        panic!("Do not directly call slack_body on views, as a view may not have a body due to it being a primitive view.")
    }
}

#[cfg(test)]
mod tests {
    use crate::slack::message::SlackMessage;

    use super::*;

    #[derive(Debug, Serialize)]
    struct DividersView;

    impl SlackView for DividersView {
        fn slack_body(&self) -> impl SlackView {
            SlackDivider.flat_chain_block(SlackDivider)
                .flat_chain_block(SlackDivider.flat_chain_block(SlackDivider.flat_chain_block(SlackDivider)))
                .flat_chain_block(SlackDivider)
        }
    }

    #[test]
    fn flat_chain_flattens_nested_dividers() {
        let message = SlackMessage::for_testing(&DividersView);
        let json = serde_json::to_string(&message).unwrap();
        let expected_json = r#"{"channel":"__TEST__","blocks":[{"type":"divider"},{"type":"divider"},{"type":"divider"},{"type":"divider"},{"type":"divider"},{"type":"divider"}]}"#;
        assert_eq!(json, expected_json)
    }

    #[derive(Debug, Serialize)]
    struct AnyDividerView;

    impl SlackView for AnyDividerView {
        fn slack_body(&self) -> impl SlackView {
            SlackDivider.erase_to_any_view().erase_to_any_view()
        }
    }

    #[test]
    fn serializing_any_view_does_not_nest() {
        let message = SlackMessage::for_testing(&AnyDividerView);
        let json = serde_json::to_string(&message).unwrap();
        let expected_json = r#"{"channel":"__TEST__","blocks":[{"type":"divider"}]}"#;
        assert_eq!(json, expected_json)
    }
}
