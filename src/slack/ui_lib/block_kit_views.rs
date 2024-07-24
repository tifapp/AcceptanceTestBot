use std::fmt::Debug;

use serde::Serialize;

use super::{primitive_view::PrimitiveView, slack_view::SlackView};

/// A section component.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct SlackSection {
    #[serde(rename = "type")]
    _type: &'static str,
    text: SlackText
}

impl SlackSection {
    /// A convenience constructor to create a section from markdown.
    pub fn from_markdown(markdown: &str) -> Self {
        Self {
            _type: "section",
            text: SlackText::markdown(markdown)
        }
    }

    pub fn empty() -> Self {
        Self::from_markdown("")
    }
}

impl SlackView for SlackSection {
    fn slack_body(&self) -> impl SlackView {
        PrimitiveView::new(self)
    }
}

/// Slack Text for use in a Section.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct SlackText {
    #[serde(rename = "type")]
    _type: &'static str,
    text: String
}

impl SlackText {
    pub fn markdown(markdown: &str) -> Self {
        Self { _type: "mrkdwn", text: markdown.to_string() }
    }

    pub fn plain(text: &str) -> Self {
        Self { _type: "plain_text", text: text.to_string() }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Clone)]
pub struct _SlackDivider {
    #[serde(rename = "type")]
    _type: &'static str
}

/// A slack divider component.
#[allow(nonstandard_style)]
pub const SlackDivider: _SlackDivider = _SlackDivider { _type: "divider" };

impl SlackView for _SlackDivider {
    fn slack_body(&self) -> impl SlackView {
        PrimitiveView::new(self)
    }
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct SlackHeader {
    #[serde(rename = "type")]
    _type: &'static str,
    text: SlackText
}

impl SlackHeader {
    pub fn new(text: &str) -> Self {
        Self { _type: "header", text: SlackText::plain(text) }
    }
}

impl SlackView for SlackHeader {
    fn slack_body(&self) -> impl SlackView {
        PrimitiveView::new(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::slack::ui_lib::test_support::assert_blocks_json;

    use super::*;

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
            TextView.flat_chain_block(SlackDivider).flat_chain_block(TextView)
        }
    }

    #[test]
    fn nested_view_flattens_to_proper_json() {
        assert_blocks_json(
            &NestedView,
            r#"[{"text":{"text":"Hello World!","type":"mrkdwn"},"type":"section"},{"type":"divider"},{"text":{"text":"Hello World!","type":"mrkdwn"},"type":"section"}]"#
        );
    }
}
