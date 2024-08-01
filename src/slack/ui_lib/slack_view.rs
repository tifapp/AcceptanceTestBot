use super::{any_view::AnySlackView, blocks::{SlackBlocks, _SlackBlocksCollection}, flat_chain_view::_FlatChainSlackView};

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
    fn flat_chain_block<Other: SlackView>(
        self,
        other: Other
    ) -> _FlatChainSlackView<Self, Other> {
        self.flat_chain_block_ref(&other)
    }

    /// Chains 2 slack view components as 2 separate blocks.
    ///
    /// `other` will be flattened when serialized into a slack message, so it is safe to call
    /// `flat_chain_block` inside the `slack_body` of `other` without incurring uneccessary nesting.
    fn flat_chain_block_ref<Other: SlackView>(
        self,
        other: &Other
    ) -> _FlatChainSlackView<Self, Other> {
        _FlatChainSlackView::new(self, other)
    }

    /// Type erases this view to an `AnySlackView`.
    fn erase_to_any_view(self) -> AnySlackView {
        AnySlackView::erasing(self)
    }

    // Do not override this method. It is an implementation detail of the library.
    fn __push_blocks_into(&self, slack_blocks: &mut _SlackBlocksCollection) {
        slack_blocks.push_view(self)
    }
}

/// Renders the specified `SlackView` into a `SlackBlocks` instance.
pub fn render_slack_view(view: &impl SlackView) -> SlackBlocks {
    let mut blocks = _SlackBlocksCollection::new();
    view.__push_blocks_into(&mut blocks);
    SlackBlocks::from(blocks)
}
