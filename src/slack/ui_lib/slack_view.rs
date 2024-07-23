use super::{any_view::AnySlackView, blocks::_SlackBlocks, flat_chain_view::_FlatChainSlackView, primitive_view::PrimitiveView};

/// A trait for implementing a slack view.
///
/// Slack views are composed of blocks that are serialized to JSON. Views are implemented using
/// the `slack_body` method, where they must return another `SlackView`.
pub trait SlackView {
    /// The content of this view.
    fn slack_body(&self) -> impl SlackView;

    /// Chains 2 slack view components as 2 separate blocks.
    ///
    /// `other` will be flattened when serialized into a slack message, so it is safe to call
    /// `flat_chain_block` inside the `slack_body` of `other` without incurring uneccessary nesting.
    fn flat_chain_block<Other: SlackView>(
        self,
        other: Other
    ) -> _FlatChainSlackView<Self, Other> where Self: Sized {
        _FlatChainSlackView::new(self, other)
    }

    /// Chains 2 slack view components as 2 separate blocks if `condition` is true.
    ///
    /// `other` will be flattened when serialized into a slack message, so it is safe to call
    /// `flat_chain_block` inside the `slack_body` of `other` without incurring uneccessary nesting.
    fn flat_chain_block_if<Other: SlackView>(
        self,
        condition: bool,
        other: impl FnOnce() -> Other
    ) -> _FlatChainSlackView<Self, Option<Other>> where Self: Sized {
        if condition {
            _FlatChainSlackView::new(self, Some(other()))
        } else {
            _FlatChainSlackView::new(self, None)
        }
    }

    /// Type erases this view to an `AnySlackView`.
    fn erase_to_any_view(self) -> AnySlackView where Self: Sized {
        AnySlackView::erasing(self)
    }

    fn _push_blocks_into(&self, slack_blocks: &mut _SlackBlocks) where Self: Sized {
        slack_blocks.push_view(self)
    }
}
