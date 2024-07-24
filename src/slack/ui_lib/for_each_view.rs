use super::{blocks::_SlackBlocks, empty_view::EmptySlackView, slack_view::SlackView};

/// A view for displaying a list of items.
pub struct ForEachView<
    Item,
    View: SlackView,
    MakeView: Fn(&Item) -> View
> {
    items: Vec<Item>,
    make_view: MakeView
}

impl <
    Item,
    View: SlackView,
    MakeView: Fn(&Item) -> View
> ForEachView<Item, View, MakeView> {
    /// Creates a new `ForEachView` using the specified iterator and view function.
    pub fn new(items: impl Iterator<Item = Item>, make_view: MakeView) -> Self {
        Self { items: items.collect(), make_view }
    }
}

impl <
    Item,
    View: SlackView,
    MakeView: Fn(&Item) -> View
> SlackView for ForEachView<Item, View, MakeView> {
    fn _push_blocks_into(&self, slack_blocks: &mut _SlackBlocks) where Self: Sized {
        for item in self.items.iter() {
            slack_blocks.push_view(&(self.make_view)(item))
        }
    }

    fn slack_body(&self) -> impl SlackView {
        EmptySlackView
    }
}
