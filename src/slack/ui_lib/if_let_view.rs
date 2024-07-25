use super::{blocks::_SlackBlocksCollection, empty_view::EmptySlackView, slack_view::SlackView};

pub struct IfLet<'v, Value, View: SlackView, MakeView: Fn(&'v Value) -> View> {
    value: Option<&'v Value>,
    make_view: MakeView
}

impl <
    'v,
    Value,
    View: SlackView,
    MakeView: Fn(&'v Value) -> View
> IfLet<'v, Value, View, MakeView> {
    pub fn some(value: Option<&'v Value>, make_view: MakeView) -> Self {
        Self { value, make_view }
    }
}

impl <
    'v,
    Value,
    View: SlackView,
    MakeView: Fn(&'v Value) -> View
> SlackView for IfLet<'v, Value, View, MakeView> {
    fn __push_blocks_into(&self, slack_blocks: &mut _SlackBlocksCollection) {
        if let Some(value) = self.value {
            (self.make_view)(value).__push_blocks_into(slack_blocks)
        }
    }

    fn slack_body(&self) -> impl SlackView {
        EmptySlackView
    }
}
