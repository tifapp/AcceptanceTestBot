use super::{blocks::_SlackBlocks, empty_view::EmptySlackView, slack_view::SlackView};

#[derive(Debug, PartialEq, Eq)]
pub struct _FlatChainSlackView<Base: SlackView, Other: SlackView> {
    base: Base,
    other: Other
}

impl <Base: SlackView, Other: SlackView> _FlatChainSlackView<Base, Other> {
    pub(super) fn new(base: Base, other: Other) -> Self {
        Self { base, other }
    }
}

impl <Base: SlackView, Other: SlackView> SlackView for _FlatChainSlackView<Base, Other> {
    fn slack_body(&self) -> impl SlackView {
        EmptySlackView
    }

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

    #[test]
    fn flat_chain_if_renders_when_true() {
        assert_blocks_json(
            &EmptySlackView.flat_chain_block_if(true, || DividersView),
            r#"[{"type":"divider"},{"type":"divider"},{"type":"divider"},{"type":"divider"},{"type":"divider"},{"type":"divider"}]"#
        );
    }

    #[test]
    fn flat_chain_if_renders_nothing_when_false() {
        assert_blocks_json(&EmptySlackView.flat_chain_block_if(false, || DividersView), r#"[]"#);
    }
}
