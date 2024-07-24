use super::{blocks::_SlackBlocksCollection, empty_view::EmptySlackView, slack_view::SlackView};

impl <View: SlackView> SlackView for Option<View> {
    fn __push_blocks_into(&self, slack_blocks: &mut _SlackBlocksCollection) where Self: Sized {
        if let Some(view) = self {
            view.__push_blocks_into(slack_blocks)
        }
    }

    fn slack_body(&self) -> impl SlackView {
        EmptySlackView
    }
}

#[cfg(test)]
mod tests {
    use crate::slack::ui_lib::{block_kit_views::SlackDivider, flat_chain_view::_FlatChainSlackView, slack_view::SlackView, test_support::assert_blocks_json};
    use super::*;

    struct TestView<Base: SlackView + Clone> {
        child: Option<Base>
    }

    impl <Base: SlackView + Clone> SlackView for TestView<Base> {
        fn slack_body(&self) -> impl SlackView {
            self.child.clone()
        }
    }

    #[test]
    fn includes_block_when_value_present() {
        let view = TestView { child: Some(SlackDivider) };
        assert_blocks_json(&view, r#"[{"type":"divider"}]"#)
    }

    #[test]
    fn excluded_block_when_value_not_present() {
        let view = TestView::<SlackDivider> { child: None };
        assert_blocks_json(&view, r#"[]"#)
    }

    #[test]
    fn includes_block_when_nested_value_present() {
        assert_blocks_json(
            &Some(SlackDivider.flat_chain_block(SlackDivider)),
            r#"[{"type":"divider"},{"type":"divider"}]"#
        )
    }

    #[test]
    fn excluded_block_when_nested_value_not_present() {
        assert_blocks_json(&None::<_FlatChainSlackView<SlackDivider, SlackDivider>>, r#"[]"#)
    }
}
