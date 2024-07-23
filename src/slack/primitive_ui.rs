use std::fmt::Debug;

use serde::{ser::SerializeSeq, Serialize, Serializer};
use serde_json::json;

/// A trait for a primitive slack component.
///
/// Implementing types should only represent primitive Block Kit slack elements. For higher level
/// views, implement `SlackView` instead.
pub trait PrimitiveSlackView: Serialize + Sized {
    fn _flat_deep_subviews(&self) -> Vec<AnySlackView> {
        vec![self.erase_to_any_view()]
    }

    /// Chains 2 slack view components, flattening their resulting block JSON.
    fn flat_chain<Other: PrimitiveSlackView>(
        self,
        other: Other
    ) -> _FlatChainSlackView<Self, Other> {
        _FlatChainSlackView { base: self, other }
    }

    /// Type erases this view.
    fn erase_to_any_view(&self) -> AnySlackView {
        AnySlackView { json: json!(self) }
    }
}

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

impl PrimitiveSlackView for SlackSection {}

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

impl PrimitiveSlackView for SlackMarkdownText {}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct _SlackDivider {
    #[serde(rename = "type")]
    _type: &'static str
}

/// A slack divider component.
#[allow(nonstandard_style)]
pub const Divider: _SlackDivider = _SlackDivider { _type: "divider" };

impl PrimitiveSlackView for _SlackDivider {}

#[derive(Debug, PartialEq, Eq)]
pub struct _FlatChainSlackView<Base: PrimitiveSlackView, Other: PrimitiveSlackView> {
    base: Base,
    other: Other
}

impl <Base: PrimitiveSlackView + 'static, Other: PrimitiveSlackView + 'static> Serialize
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

impl <Base: PrimitiveSlackView + 'static, Other: PrimitiveSlackView + 'static> PrimitiveSlackView
    for _FlatChainSlackView<Base, Other> {

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

impl Serialize for AnySlackView {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        self.json.serialize(serializer)
    }
}

impl PrimitiveSlackView for AnySlackView {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_chain_flattens_nested_dividers() {
        let dividers = Divider.flat_chain(Divider)
            .flat_chain(Divider.flat_chain(Divider.flat_chain(Divider)))
            .flat_chain(Divider);
        let json = serde_json::to_string(&dividers).unwrap();
        let expected_json = r#"[{"type":"divider"},{"type":"divider"},{"type":"divider"},{"type":"divider"},{"type":"divider"},{"type":"divider"}]"#;
        assert_eq!(json, expected_json)
    }

    #[test]
    fn serializing_any_view_does_not_nest() {
        let divider = Divider.erase_to_any_view().erase_to_any_view();
        let json = serde_json::to_string(&divider).unwrap();
        let expected_json = r#"{"type":"divider"}"#;
        assert_eq!(json, expected_json)
    }
}
