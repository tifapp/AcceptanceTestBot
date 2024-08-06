use std::fmt::Debug;

use serde::{ser::SerializeStruct, Serialize};

use super::{primitive_view::PrimitiveView, slack_view::SlackView};

/// A section component.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct SlackSection {
    #[serde(rename = "type")]
    _type: &'static str,
    text: SlackText,
}

impl SlackSection {
    /// A convenience constructor to create a section from markdown.
    pub fn from_markdown(markdown: &str) -> Self {
        Self {
            _type: "section",
            text: SlackText::markdown(markdown),
        }
    }

    /// A convenience constructor to create a section from plain text.
    pub fn from_plaintext(text: &str) -> Self {
        Self {
            _type: "section",
            text: SlackText::plain(text),
        }
    }

    /// Constructs an empty section.
    pub fn empty() -> Self {
        Self::from_markdown(" ")
    }
}

impl SlackView for SlackSection {
    fn slack_body(&self) -> impl SlackView {
        PrimitiveView::new(self)
    }
}

impl SlackSection {
    /// Enables slack emojis on this section if `is_enabled` is true.
    ///
    /// This has no effect on markdown text.
    pub fn emoji_enabled(self, is_enabled: bool) -> Self {
        Self {
            text: self.text.emoji_enabled(is_enabled),
            ..self
        }
    }
}

/// Slack Text for use in a Section.
#[derive(Debug, PartialEq, Eq)]
pub struct SlackText {
    _type: &'static str,
    text: String,
    emoji: bool,
}

impl SlackText {
    pub fn markdown(markdown: &str) -> Self {
        Self {
            _type: "mrkdwn",
            text: markdown.to_string(),
            emoji: true,
        }
    }

    pub fn plain(text: &str) -> Self {
        Self {
            _type: "plain_text",
            text: text.to_string(),
            emoji: true,
        }
    }
}

impl SlackText {
    /// Enables slack emojis on this text if `is_enabled` is true.
    ///
    /// This has no effect on markdown text.
    pub fn emoji_enabled(self, is_enabled: bool) -> Self {
        Self {
            emoji: is_enabled,
            ..self
        }
    }

    fn is_markdown(&self) -> bool {
        self._type == "mrkdwn"
    }
}

impl Serialize for SlackText {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state =
            serializer.serialize_struct("SlackText", if self.is_markdown() { 2 } else { 3 })?;
        if !self.is_markdown() && !self.emoji {
            state.serialize_field("emoji", &self.emoji)?
        }
        state.serialize_field("text", &self.text)?;
        state.serialize_field("type", &self._type)?;
        state.end()
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Clone)]
pub struct SlackDivider {
    #[serde(rename = "type")]
    _type: &'static str,
}

/// A slack divider component.
#[allow(nonstandard_style)]
pub const SlackDivider: SlackDivider = SlackDivider { _type: "divider" };

impl SlackView for SlackDivider {
    fn slack_body(&self) -> impl SlackView {
        PrimitiveView::new(self)
    }
}

/// A Slack Header view.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct SlackHeader {
    #[serde(rename = "type")]
    _type: &'static str,
    text: SlackText,
}

impl SlackHeader {
    pub fn new(text: &str) -> Self {
        Self {
            _type: "header",
            text: SlackText::plain(text),
        }
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
            TextView
                .flat_chain_block(SlackDivider)
                .flat_chain_block(TextView)
        }
    }

    #[test]
    fn nested_view_flattens_to_proper_json() {
        assert_blocks_json(
            &NestedView,
            r#"[{"text":{"text":"Hello World!","type":"mrkdwn"},"type":"section"},{"type":"divider"},{"text":{"text":"Hello World!","type":"mrkdwn"},"type":"section"}]"#,
        );
    }
}
