use crate::{language::ast::RoswaalTestSyntax, location::location::RoswaalStringLocations};

use super::pull_request::GithubPullRequest;

/// A struct containing neccessary metadata for operating in a roswaal compatible git repo.
pub struct RoswaalGitRepoMetadata {
    repo_root_dir_path: String,
    test_cases_root_dir_path: String,
    add_test_cases_pr: fn(
        test_names_with_syntax: Vec<(&str, RoswaalTestSyntax)>,
        head_branch: String
    ) -> GithubPullRequest,
    locations_path: String,
    add_locations_pr: fn(RoswaalStringLocations, head_branch: String) -> GithubPullRequest
}

impl RoswaalGitRepoMetadata {
    /// Metadata for the main frontend repo.
    pub fn for_tif_react_frontend() -> Self {
        Self {
            repo_root_dir_path: "./FitnessProject".to_string(),
            test_cases_root_dir_path: "./FitnessProject/roswaal".to_string(),
            add_test_cases_pr: GithubPullRequest::for_test_cases_tif_react_frontend,
            locations_path: "./FitnessProject/rosswaal/Locations.ts".to_string(),
            add_locations_pr: GithubPullRequest::for_locations_tif_react_frontend
        }
    }

    /// Metadata for a local testing repo.
    pub fn for_testing() -> Self {
        Self {
            repo_root_dir_path: "./FitnessProjectTests".to_string(),
            test_cases_root_dir_path: "./FitnessProjectTests/roswaal".to_string(),
            add_test_cases_pr: |cases, head_branch| {
                GithubPullRequest::for_test_cases_tif_react_frontend(cases, head_branch)
                    .for_testing_do_not_merge()
            },
            locations_path: "./FitnessProjectTests/rosswaal/Locations.ts".to_string(),
            add_locations_pr: |locations, head_branch| {
                GithubPullRequest::for_locations_tif_react_frontend(locations, head_branch)
                    .for_testing_do_not_merge()
            }
        }
    }
}
