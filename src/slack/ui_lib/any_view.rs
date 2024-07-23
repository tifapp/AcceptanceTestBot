use super::{blocks::_SlackBlocks, primitive_view::_PrimitiveView, slack_view::{find_primitive_view, SlackView}};

/// A type-erased view.
pub struct AnySlackView {
    inner: _PrimitiveView
}

impl AnySlackView {
    /// Erases the specified view.
    pub fn erasing(view: impl SlackView) -> Self {
        Self { inner: find_primitive_view(&view) }
    }

    /// Erases a reference to the specified view.
    pub fn erasing_ref(view: &impl SlackView) -> Self {
        Self { inner: find_primitive_view(view) }
    }
}

impl SlackView for AnySlackView {
    fn _as_primitive_view(&self) -> Option<_PrimitiveView> {
        Some(self.inner.clone())
    }

    fn _push_blocks_into(&self, slack_blocks: &mut _SlackBlocks) where Self: Sized {
        slack_blocks.push_primitive_view(&self.inner)
    }

    fn slack_body(&self) -> impl SlackView {
        self.inner.slack_body()
    }
}
