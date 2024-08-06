use std::env;

use super::{branch_name::RoswaalOwnedGitBranchName, pull_request::GithubPullRequest};
use crate::{
    language::{ast::RoswaalTestSyntax, compilation_results::RoswaalTestCompilationResults},
    location::location::RoswaalStringLocations,
    tests_data::query::RoswaalTestNamesString,
    utils::string::ToAsciiKebabCase,
};

/// A struct containing neccessary metadata for operating in a roswaal compatible git repo.
#[derive(Debug, Clone)]
pub struct RoswaalGitRepositoryMetadata {
    base_branch_name: String,
    repo_root_dir_path: String,
    ssh_private_key_home_path: String,
    test_cases_root_dir_path: String,
    add_test_cases_pr: fn(
        results: &RoswaalTestCompilationResults,
        &RoswaalOwnedGitBranchName,
    ) -> GithubPullRequest,
    locations_path: String,
    add_locations_pr: fn(&RoswaalStringLocations, &RoswaalOwnedGitBranchName) -> GithubPullRequest,
    remove_tests_pr: fn(&RoswaalTestNamesString, &RoswaalOwnedGitBranchName) -> GithubPullRequest,
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
            add_locations_pr: GithubPullRequest::for_locations_tif_react_frontend,
            remove_tests_pr: GithubPullRequest::for_removing_test_cases_tif_react_frontend,
        }
    }

    /// Metadata for a local testing repo.
    pub fn for_testing() -> Self {
        Self::for_testing_with_custom_base_branch(TEST_REPO_BASE_BRANCH_NAME)
    }

    /// Metadata for a local testing repo with a custom base branch name.
    pub fn for_testing_with_custom_base_branch(base_branch_name: &str) -> Self {
        Self {
            base_branch_name: base_branch_name.to_string(),
            repo_root_dir_path: "./FitnessProjectTest".to_string(),
            ssh_private_key_home_path: env::var("TEST_SSH_PRIVATE_KEY_HOME_PATH")
                .expect("Ensure to the set the TEST_SSH_PRIVATE_KEY_HOME_PATH variable in your .env file to the home path of your private ssh-key (Ex. ./.ssh/id_rsa)"),
            test_cases_root_dir_path: "./FitnessProjectTest/roswaal".to_string(),
            add_test_cases_pr: |cases, head_branch| {
                GithubPullRequest::for_test_cases_tif_react_frontend(cases, head_branch)
                    .for_testing_do_not_merge()
            },
            locations_path: "./FitnessProjectTest/roswaal/Locations.ts".to_string(),
            add_locations_pr: |locations, head_branch| {
                GithubPullRequest::for_locations_tif_react_frontend(locations, head_branch)
                    .for_testing_do_not_merge()
            },
            remove_tests_pr: |test_names, head_branch| {
                GithubPullRequest::for_removing_test_cases_tif_react_frontend(
                    test_names,
                    head_branch
                )
                .for_testing_do_not_merge()
            }
        }
    }
}

pub const TEST_REPO_BASE_BRANCH_NAME: &str = "main";

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

    pub fn add_locations_pull_request(
        &self,
        locations: &RoswaalStringLocations,
        branch_name: &RoswaalOwnedGitBranchName,
    ) -> GithubPullRequest {
        (self.add_locations_pr)(locations, branch_name)
    }

    pub fn add_tests_pull_request(
        &self,
        results: &RoswaalTestCompilationResults,
        branch_name: &RoswaalOwnedGitBranchName,
    ) -> GithubPullRequest {
        (self.add_test_cases_pr)(results, branch_name)
    }

    pub fn remove_tests_pull_request<'a>(
        &self,
        test_names: &RoswaalTestNamesString,
        branch_name: &RoswaalOwnedGitBranchName,
    ) -> GithubPullRequest {
        (self.remove_tests_pr)(test_names, branch_name)
    }

    pub fn test_dirpath(&self, test_name: &str) -> String {
        let name = test_name.to_ascii_kebab_case().to_ascii_lowercase();
        format!("{}/{}", self.test_cases_root_dir_path, name)
    }
}
