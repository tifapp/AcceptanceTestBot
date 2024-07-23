use serde::Serialize;

use super::primitive_views::{AnySlackView, _FlatChainSlackView};

/// A trait for implementing a slack view.
///
/// Slack views are composed of blocks that are serialized to JSON. Views are implemented using
/// the `slack_body` method, where they must return another `SlackView`.
pub trait SlackView: Serialize + Sized {
    /// Returns another `SlackView` based on the content of this view.
    fn slack_body(&self) -> impl SlackView;

    /// Chains 2 slack view components as 2 separate blocks.
    ///
    /// `other` will be flattened when serialized into a slack message, so it is safe to call
    /// `flat_chain_block` inside the `slack_body` of `other` without incurring uneccessary nesting.
    fn flat_chain_block<Other: SlackView + 'static>(
        self,
        other: Other
    ) -> _FlatChainSlackView<Self, Other> where Self: 'static {
        _FlatChainSlackView::new(self, other)
    }

    /// Type erases this view.
    fn erase_to_any_view(&self) -> AnySlackView {
        AnySlackView::from(self)
    }

    fn _flat_deep_subviews(&self) -> Vec<AnySlackView> {
        vec![self.erase_to_any_view()]
    }
}
