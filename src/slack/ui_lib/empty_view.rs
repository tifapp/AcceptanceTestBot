use super::{any_view::AnySlackView, blocks::_SlackBlocksCollection, primitive_view::PrimitiveView, slack_view::SlackView};

/// A view to use as a placeholder when no other view can be returned from the `slack_body` method
/// of another view.
pub struct EmptySlackView;

impl SlackView for EmptySlackView {
    fn __push_blocks_into(&self, _: &mut _SlackBlocksCollection) where Self: Sized {
    }

    fn erase_to_any_view(self) -> AnySlackView {
        AnySlackView::from(_SlackBlocksCollection::new())
    }

    fn slack_body(&self) -> impl SlackView {
        PrimitiveView::empty()
    }
}

#[cfg(test)]
mod tests {
    use crate::slack::ui_lib::{block_kit_views::SlackSection, slack_view::SlackView, test_support::assert_blocks_json};

    use super::EmptySlackView;

    struct View;

    impl SlackView for View {
        fn slack_body(&self) -> impl SlackView {
            SlackSection::from_markdown("I am bob!")
        }
    }

    struct ChainWithEmptyView;

    impl SlackView for ChainWithEmptyView {
        fn slack_body(&self) -> impl SlackView {
            View.flat_chain_block(EmptySlackView)
        }
    }

    #[test]
    fn does_not_insert_block() {
        assert_blocks_json(
            &ChainWithEmptyView,
            r#"[{"text":{"text":"I am bob!","type":"mrkdwn"},"type":"section"}]"#
        )
    }
}
