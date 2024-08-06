use std::marker::PhantomData;

use super::{
    any_view::AnySlackView, blocks::_SlackBlocksCollection, empty_view::EmptySlackView,
    slack_view::SlackView,
};

#[derive(Debug, PartialEq, Eq)]
pub struct _FlatChainSlackView<Base: SlackView, Other: SlackView> {
    blocks: _SlackBlocksCollection,
    p1: PhantomData<Base>,
    p2: PhantomData<Other>,
}

impl<Base: SlackView, Other: SlackView> _FlatChainSlackView<Base, Other> {
    pub(super) fn new(base: Base, other: &Other) -> Self {
        let mut blocks = _SlackBlocksCollection::new();
        base.__push_blocks_into(&mut blocks);
        other.__push_blocks_into(&mut blocks);
        Self::from(blocks)
    }

    fn from(blocks: _SlackBlocksCollection) -> Self {
        Self {
            blocks,
            p1: PhantomData,
            p2: PhantomData,
        }
    }
}

impl<Base: SlackView, Other: SlackView> SlackView for _FlatChainSlackView<Base, Other> {
    fn slack_body(&self) -> impl SlackView {
        EmptySlackView
    }

    fn __push_blocks_into(&self, slack_blocks: &mut _SlackBlocksCollection) {
        slack_blocks.extend(&self.blocks)
    }

    fn flat_chain_block_ref<View: SlackView>(
        mut self,
        other: &View,
    ) -> _FlatChainSlackView<Self, View> {
        other.__push_blocks_into(&mut self.blocks);
        _FlatChainSlackView::from(self.blocks)
    }

    fn erase_to_any_view(self) -> AnySlackView {
        AnySlackView::from(self.blocks)
    }
}

#[cfg(test)]
mod tests {
    use crate::slack::ui_lib::{block_kit_views::SlackDivider, test_support::assert_blocks_json};

    use super::*;

    struct DividersView;

    impl SlackView for DividersView {
        fn slack_body(&self) -> impl SlackView {
            SlackDivider
                .flat_chain_block(SlackDivider)
                .flat_chain_block(
                    SlackDivider.flat_chain_block(SlackDivider.flat_chain_block(SlackDivider)),
                )
                .flat_chain_block(SlackDivider)
        }
    }

    #[test]
    fn flat_chain_flattens_nested_dividers() {
        assert_blocks_json(
            &DividersView,
            r#"[{"type":"divider"},{"type":"divider"},{"type":"divider"},{"type":"divider"},{"type":"divider"},{"type":"divider"}]"#,
        );
    }
}
