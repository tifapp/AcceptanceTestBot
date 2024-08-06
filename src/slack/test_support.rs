use std::{fs::File, io::Read};

use serde::Deserialize;

use crate::git::branch_name::RoswaalOwnedGitBranchName;

/// Branche names to use in Slack UI snapshot tests.
///
/// `RoswaalOwnedGitBranchName` generates a unique name for each new branch. To prevent misuse of
/// the struct, we can make constant branch names by loading them from a JSON file.
#[derive(Debug, Deserialize)]
pub struct SlackTestConstantBranches {
    add_locations: RoswaalOwnedGitBranchName,
    add_tests: RoswaalOwnedGitBranchName,
    remove_tests: RoswaalOwnedGitBranchName,
}

impl SlackTestConstantBranches {
    /// Loads the branches from the test JSON file.
    pub fn load() -> Self {
        let file = File::open("./slack-test-branch-names.json").unwrap();
        serde_json::from_reader(file).unwrap()
    }

    pub fn add_locations(&self) -> &RoswaalOwnedGitBranchName {
        &self.add_locations
    }

    pub fn add_tests(&self) -> &RoswaalOwnedGitBranchName {
        &self.add_tests
    }

    pub fn remove_tests(&self) -> &RoswaalOwnedGitBranchName {
        &self.remove_tests
    }
}
