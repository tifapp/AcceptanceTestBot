use std::{fs::File, io::Read};

use serde::Deserialize;

use crate::git::branch_name::RoswaalOwnedGitBranchName;

#[derive(Debug, Deserialize)]
pub struct SlackTestConstantBranches {
    pub addLocations: RoswaalOwnedGitBranchName,
    pub addTests: RoswaalOwnedGitBranchName,
    pub removeTests: RoswaalOwnedGitBranchName
}

impl SlackTestConstantBranches {
    pub fn load() -> Self {
        let file = File::open("./slack-test-branch-names.json").unwrap();
        serde_json::from_reader(file).unwrap()
    }
}
