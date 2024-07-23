use super::{blocks::{SlackBlocks, _SlackBlocks}, empty_view::EmptySlackView, slack_view::SlackView};

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
        Self { blocks: SlackBlocks::render(view) }
    }
}

impl SlackView for AnySlackView {
    fn _push_blocks_into(&self, slack_blocks: &mut _SlackBlocks) where Self: Sized {
        slack_blocks.extend(&self.blocks)
    }

    fn slack_body(&self) -> impl SlackView {
        EmptySlackView
    }
}
