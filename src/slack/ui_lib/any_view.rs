use super::{blocks::{SlackBlocks, _SlackBlocksCollection}, empty_view::EmptySlackView, slack_view::{render_slack_view, SlackView}};

/// A type-erased view.
pub struct AnySlackView {
    blocks: SlackBlocks
}

impl AnySlackView {
    /// Erases the specified view.
    pub fn erasing(view: impl SlackView) -> Self {
        Self::erasing_ref(&view)
    }

    /// Erases a reference to the specified view.
    pub fn erasing_ref(view: &impl SlackView) -> Self {
        Self { blocks: render_slack_view(view) }
    }

    pub(super) fn from(collection: _SlackBlocksCollection) -> Self {
        Self { blocks: SlackBlocks::from(collection) }
    }
}

impl SlackView for AnySlackView {
    fn __push_blocks_into(&self, slack_blocks: &mut _SlackBlocksCollection) where Self: Sized {
        slack_blocks.extend(&self.blocks.collection())
    }

    fn slack_body(&self) -> impl SlackView {
        EmptySlackView
    }
}
