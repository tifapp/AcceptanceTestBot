use super::{blocks::_SlackBlocks, flat_chain_view::_FlatChainSlackView};

/// A trait for implementing a slack view.
///
/// Slack views are composed of blocks that are serialized to JSON. Views are implemented using
/// the `slack_body` method, where they must return another `SlackView`.
pub trait SlackView: Sized {
    /// The content of this view.
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

    fn _push_blocks_into(&self, slack_blocks: &mut _SlackBlocks) {
        slack_blocks.push_view(self)
    }
}
