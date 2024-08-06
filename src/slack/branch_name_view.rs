use crate::git::branch_name::RoswaalOwnedGitBranchName;

use super::ui_lib::{block_kit_views::SlackSection, if_let_view::IfLet, slack_view::SlackView};

pub struct OptionalBranchNameView<'v> {
    branch_name: Option<&'v RoswaalOwnedGitBranchName>,
}

impl<'v> OptionalBranchNameView<'v> {
    pub fn new(branch_name: Option<&'v RoswaalOwnedGitBranchName>) -> Self {
        Self { branch_name }
    }
}

impl<'v> SlackView for OptionalBranchNameView<'v> {
    fn slack_body(&self) -> impl SlackView {
        IfLet::some(self.branch_name, |branch_name| {
            let name = format!("_(Branch: {})_", branch_name.to_string());
            SlackSection::from_markdown(&name)
        })
    }
}
