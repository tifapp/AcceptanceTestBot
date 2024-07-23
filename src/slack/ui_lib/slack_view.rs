use super::{blocks::_SlackBlocks, flat_chain_view::_FlatChainSlackView, primitive_view::_PrimitiveView};

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

    fn _push_blocks_into(&self, slack_blocks: &mut _SlackBlocks) where Self: Sized {
        slack_blocks.push_view(self)
    }

    fn _as_primitive_view(&self) -> Option<_PrimitiveView> {
        None
    }
}

pub(super) fn find_primitive_view(view: &impl SlackView) -> _PrimitiveView {
    if let Some(view) = view._as_primitive_view() {
        return view
    }
    find_primitive_view(&view.slack_body())
}
