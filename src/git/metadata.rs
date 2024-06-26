use std::env;

use crate::{language::ast::RoswaalTestSyntax, location::location::RoswaalStringLocations};
use super::{branch_name::RoswaalOwnedGitBranchName, pull_request::GithubPullRequest};

/// A struct containing neccessary metadata for operating in a roswaal compatible git repo.
#[derive(Debug, Clone)]
pub struct RoswaalGitRepositoryMetadata {
    base_branch_name: String,
    repo_root_dir_path: String,
    ssh_private_key_home_path: String,
    test_cases_root_dir_path: String,
    add_test_cases_pr: fn(
        test_names_with_syntax: &Vec<(&str, RoswaalTestSyntax)>,
        &RoswaalOwnedGitBranchName
    ) -> GithubPullRequest,
    locations_path: String,
    add_locations_pr: fn(
        &RoswaalStringLocations,
        &RoswaalOwnedGitBranchName
    ) -> GithubPullRequest
}

impl RoswaalGitRepositoryMetadata {
    /// Metadata for the main frontend repo.
    pub fn for_tif_react_frontend() -> Self {
        Self {
            base_branch_name: "development".to_string(),
            repo_root_dir_path: "./FitnessProject".to_string(),
            ssh_private_key_home_path: "./.ssh/id_rsa".to_string(),
            test_cases_root_dir_path: "./FitnessProject/roswaal".to_string(),
            add_test_cases_pr: GithubPullRequest::for_test_cases_tif_react_frontend,
            locations_path: "./FitnessProject/roswaal/Locations.ts".to_string(),
            add_locations_pr: GithubPullRequest::for_locations_tif_react_frontend
        }
    }

    /// Metadata for a local testing repo.
    pub fn for_testing() -> Self {
        Self {
            base_branch_name: "main".to_string(),
            repo_root_dir_path: "./FitnessProjectTest".to_string(),
            ssh_private_key_home_path: "./.ssh/id_mhayes".to_string(),
            test_cases_root_dir_path: "./FitnessProjectTest/roswaal".to_string(),
            add_test_cases_pr: |cases, head_branch| {
                GithubPullRequest::for_test_cases_tif_react_frontend(cases, head_branch)
                    .for_testing_do_not_merge()
            },
            locations_path: "./FitnessProjectTest/roswaal/Locations.ts".to_string(),
            add_locations_pr: |locations, head_branch| {
                GithubPullRequest::for_locations_tif_react_frontend(locations, head_branch)
                    .for_testing_do_not_merge()
            }
        }
    }
}

impl RoswaalGitRepositoryMetadata {
    /// Returns the name of the branch that changes are primarily merged to (eg. development).
    pub fn base_branch_name(&self) -> &str {
        &self.base_branch_name
    }

    /// Returns a string path to the private ssh key to use when pushing and pulling changes from
    /// the remote repository.
    pub fn ssh_private_key_path(&self) -> String {
        let home = env::var("HOME").unwrap();
        format!("{}/{}", home, self.ssh_private_key_home_path)
    }

    /// Returns a string path relative to the root directory of the repository.
    pub fn relative_path(&self, path: &str) -> String {
        format!("{}/{}", self.repo_root_dir_path, path)
    }

    /// Returns the path to the locations file.
    pub fn locations_path(&self) -> &str {
        &self.locations_path
    }
}
