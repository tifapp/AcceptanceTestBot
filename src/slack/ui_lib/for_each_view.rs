use super::{empty_view::EmptySlackView, flat_chain_view::_FlatChainSlackView, slack_view::SlackView};

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
    fn slack_body(&self) -> impl SlackView {
        _FlatChainSlackView::new_with_others(
            EmptySlackView,
            self.items.iter().map(|i| (self.make_view)(i)).collect()
        )
    }
}
