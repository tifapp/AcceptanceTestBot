use super::{empty_view::EmptySlackView, flat_chain_view::_FlatChainSlackView, slack_view::SlackView};

/// A view that conditionally renders `View`.
pub struct If<View: SlackView, MakeView: Fn() -> View> {
    condition: bool,
    make_view: MakeView
}

impl <View: SlackView, MakeView: Fn() -> View> If<View, MakeView> {
    pub fn is_true(condition: bool, make_view: MakeView) -> Self {
        Self { condition, make_view }
    }
}

impl <View: SlackView, MakeView: Fn() -> View> SlackView for If<View, MakeView> {
    fn slack_body(&self) -> impl SlackView {
        if self.condition {
            Some((self.make_view)())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::slack::ui_lib::{block_kit_views::SlackDivider, slack_view::SlackView, test_support::assert_blocks_json};

    use super::If;

    struct DividersView;

    impl SlackView for DividersView {
        fn slack_body(&self) -> impl SlackView {
            SlackDivider
        }
    }

    #[test]
    fn flat_chain_if_renders_when_true() {
        assert_blocks_json(&If::is_true(true, || DividersView), r#"[{"type":"divider"}]"#);
    }

    #[test]
    fn flat_chain_if_renders_nothing_when_false() {
        assert_blocks_json(&If::is_true(false, || DividersView), r#"[]"#);
    }
}
