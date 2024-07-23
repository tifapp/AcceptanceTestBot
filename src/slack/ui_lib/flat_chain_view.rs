use super::{blocks::_SlackBlocks, primitive_view::PrimitiveView, slack_view::SlackView};

#[derive(Debug, PartialEq, Eq)]
pub struct _FlatChainSlackView<Base: SlackView, Other: SlackView> {
    base: Base,
    other: Other
}

impl <Base: SlackView + 'static, Other: SlackView + 'static>
    _FlatChainSlackView<Base, Other> {
    pub(super) fn new(base: Base, other: Other) -> Self {
        Self { base, other }
    }
}

impl <Base: SlackView + 'static, Other: SlackView + 'static> SlackView
    for _FlatChainSlackView<Base, Other> {
    fn slack_body(&self) -> impl SlackView { PrimitiveView::empty() }

    fn _push_blocks_into(&self, slack_blocks: &mut _SlackBlocks) {
        self.base._push_blocks_into(slack_blocks);
        self.other._push_blocks_into(slack_blocks)
    }
}

#[cfg(test)]
mod tests {
    use crate::slack::ui_lib::{block_kit_views::SlackDivider, test_support::assert_blocks_json};

    use super::*;

    struct DividersView;

    impl SlackView for DividersView {
        fn slack_body(&self) -> impl SlackView {
            SlackDivider.flat_chain_block(SlackDivider)
                .flat_chain_block(
                    SlackDivider.flat_chain_block(
                        SlackDivider.flat_chain_block(SlackDivider)
                    )
                )
                .flat_chain_block(SlackDivider)
        }
    }

    #[test]
    fn flat_chain_flattens_nested_dividers() {
        assert_blocks_json(
            &DividersView,
            r#"[{"type":"divider"},{"type":"divider"},{"type":"divider"},{"type":"divider"},{"type":"divider"},{"type":"divider"}]"#
        );
    }
}
