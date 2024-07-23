use super::{any_view::AnySlackView, empty_view::EmptySlackView, slack_view::SlackView};

impl <View: SlackView> SlackView for Option<&View> {
    fn slack_body(&self) -> impl SlackView {
        if let Some(view) = self {
            return AnySlackView::erasing_ref(view.to_owned())
        }
        AnySlackView::erasing(EmptySlackView)
    }
}

#[cfg(test)]
mod tests {
    use crate::slack::ui_lib::{block_kit_views::{SlackDivider, _SlackDivider}, slack_view::SlackView, test_support::assert_blocks_json};
    use super::*;

    struct TestView<Base: SlackView> {
        child: Option<Base>
    }

    impl <Base: SlackView> SlackView for TestView<Base> {
        fn slack_body(&self) -> impl SlackView {
            self.child.as_ref()
        }
    }

    #[test]
    fn includes_block_when_value_present() {
        let view = TestView { child: Some(SlackDivider) };
        assert_blocks_json(&view, r#"[{"type":"divider"}]"#)
    }

    #[test]
    fn excluded_block_when_value_present() {
        let view = TestView::<_SlackDivider> { child: None };
        assert_blocks_json(&view, r#"[]"#)
    }
}
