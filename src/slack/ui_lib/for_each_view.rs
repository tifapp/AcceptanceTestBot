use super::{blocks::_SlackBlocksCollection, empty_view::EmptySlackView, slack_view::SlackView};

/// A view for displaying a list of items.
pub struct ForEachView<Item, View: SlackView, MakeView: Fn(&Item) -> View> {
    items: Vec<Item>,
    make_view: MakeView,
}

impl<Item, View: SlackView, MakeView: Fn(&Item) -> View> ForEachView<Item, View, MakeView> {
    /// Creates a new `ForEachView` using the specified iterator and view function.
    pub fn new(items: impl Iterator<Item = Item>, make_view: MakeView) -> Self {
        Self {
            items: items.collect(),
            make_view,
        }
    }
}

impl<Item, View: SlackView, MakeView: Fn(&Item) -> View> SlackView
    for ForEachView<Item, View, MakeView>
{
    fn __push_blocks_into(&self, slack_blocks: &mut _SlackBlocksCollection)
    where
        Self: Sized,
    {
        for item in self.items.iter() {
            (self.make_view)(item).__push_blocks_into(slack_blocks);
        }
    }

    fn slack_body(&self) -> impl SlackView {
        EmptySlackView
    }
}

#[cfg(test)]
mod tests {
    use crate::slack::ui_lib::{
        block_kit_views::SlackDivider, slack_view::SlackView, test_support::assert_blocks_json,
    };

    use super::ForEachView;

    #[test]
    fn for_each_with_chain() {
        let view = ForEachView::new((0..2).into_iter(), |_| {
            SlackDivider.flat_chain_block(SlackDivider)
        });
        assert_blocks_json(
            &view,
            r#"[{"type":"divider"},{"type":"divider"},{"type":"divider"},{"type":"divider"}]"#,
        )
    }
}
