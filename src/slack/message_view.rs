use crate::utils::env::RoswaalEnvironement;

use super::ui_lib::{block_kit_views::SlackSection, if_view::If, slack_view::SlackView};

pub struct MessageView<'v, Base: SlackView> {
    base: &'v Base,
}

impl<'v, Base: SlackView> MessageView<'v, Base> {
    pub fn new(base: &'v Base) -> Self {
        Self { base }
    }
}

impl<'v, Base: SlackView> SlackView for MessageView<'v, Base> {
    fn slack_body(&self) -> impl SlackView {
        If::is_true(
            RoswaalEnvironement::current() == RoswaalEnvironement::Dev,
            || {
                SlackSection::from_markdown(
                    "_This message was sent for development purposeeeeeeeeeees. Please Ignoooooooore!_",
                )
            },
        )
        .flat_chain_block_ref(self.base)
    }
}
