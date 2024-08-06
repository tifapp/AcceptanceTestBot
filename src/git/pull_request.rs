use std::{env, future::Future};

use anyhow::Result;
use reqwest::{
    header::{CONTENT_TYPE, USER_AGENT},
    Client,
};
use serde::Serialize;

use crate::{
    language::ast::RoswaalTestSyntax,
    location::location::{RoswaalLocationStringError, RoswaalStringLocations},
    tests_data::query::RoswaalTestNamesString,
};

use super::branch_name::RoswaalOwnedGitBranchName;

/// A serializeable type for a pull request on github.
#[derive(Debug, PartialEq, Eq, Serialize, Clone)]
pub struct GithubPullRequest {
    title: String,
    body: String,
    #[serde(skip)]
    owner: String,
    #[serde(skip)]
    repo: String,
    head: RoswaalOwnedGitBranchName,
    base: String,
}

impl GithubPullRequest {
    /// Creates a PR for the main frontend repo.
    pub fn for_tif_react_frontend(
        title: &str,
        body: &str,
        head_branch: &RoswaalOwnedGitBranchName,
    ) -> Self {
        Self {
            body: format!(
                "{}

## Tickets

TASK_UNTRACKED
",
                body
            ),
            title: format!("Roswaal: {}", title),
            owner: "tifapp".to_string(),
            repo: "FitnessProject".to_string(),
            base: "development".to_string(),
            head: head_branch.clone(),
        }
    }

    /// Creates a PR associated with adding new locations to the main frontend repo.
    pub fn for_locations_tif_react_frontend(
        string_locations: &RoswaalStringLocations,
        head_branch: &RoswaalOwnedGitBranchName,
    ) -> Self {
        let title = format!(
            "Add Locations ({})",
            string_locations.raw_names().join(", ")
        );
        let mut body =
            "Adds the following locations to the acceptance teeeeeeeeeests:\n".to_string();
        for location in string_locations.locations() {
            let line = format!(
                "- **{}** (Latitude: {:.16}, Longitude: {:.16})\n",
                location.name().raw_name(),
                location.coordinate().latitude(),
                location.coordinate().longitude()
            );
            body.push_str(&line)
        }
        let errors = string_locations.errors();
        if !errors.is_empty() {
            body.push_str("\nThe following locations were specified in the slack command, but are invaaaaaaaalid:\n");
            for error in errors {
                body.push_str(&format!("- **{}** ", error.raw_associated_location_name()));
                match error {
                    RoswaalLocationStringError::InvalidName(_, _) => {
                        body.push_str("(Invalid Name)")
                    }
                    RoswaalLocationStringError::InvalidCoordinate { name: _ } => {
                        body.push_str("(Invalid Coordinate)")
                    }
                };
                body.push_str("\n")
            }
        }
        Self::for_tif_react_frontend(&title, &body, head_branch)
    }

    /// Creates a PR for test case creation on the frontend repo.
    pub fn for_test_cases_tif_react_frontend<'a, 'b>(
        test_names_with_syntax: &Vec<(&'a str, &RoswaalTestSyntax<'b>)>,
        head_branch: &RoswaalOwnedGitBranchName,
    ) -> Self {
        let joined_names = test_names_with_syntax
            .iter()
            .map(|(name, _)| name.to_string())
            .collect::<Vec<String>>()
            .join("\", \"");
        let title = format!("Add Tests \"{}\"", joined_names);
        let joined_test_cases = test_names_with_syntax
            .iter()
            .map(|(_, syntax)| format!("```\n{}\n```", syntax.source_code()))
            .collect::<Vec<String>>()
            .join("\n");
        let body = format!("Adds the following teeeeeests!\n\n{}", joined_test_cases);
        Self::for_tif_react_frontend(&title, &body, head_branch)
    }

    /// Creates a PR for removing test cases on the frontend repo.
    pub fn for_removing_test_cases_tif_react_frontend(
        test_names: &RoswaalTestNamesString<'_>,
        head_branch: &RoswaalOwnedGitBranchName,
    ) -> Self {
        let title = format!(
            "Remove Tests {}",
            test_names.iter().collect::<Vec<&str>>().join(", ")
        );
        let test_names_list = test_names
            .iter()
            .map(|n| format!("- {}", n))
            .collect::<Vec<String>>()
            .join("\n");
        let body = format!("Removes the following teeeeeeeests!\n{}", test_names_list);
        Self::for_tif_react_frontend(&title, &body, &head_branch)
    }
}

impl GithubPullRequest {
    /// Returns the title of this PR.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the head branch of this PR.
    pub fn head_branch(&self) -> &RoswaalOwnedGitBranchName {
        &self.head
    }

    /// Designates this PR specifically for testing and adjusts the title and body to disclaim
    /// that it should not be merged.
    ///
    /// This is useful for E2E tests.
    pub fn for_testing_do_not_merge(self) -> Self {
        Self {
            title: format!("[Test - DO NOT MERGE] {}", self.title),
            body: format!(
                "This is a test PR, please do not meeeeeeerge!!!\n\n{}",
                self.body
            ),
            owner: "roswaaltifbot".to_string(),
            repo: "FitnessProjectTest".to_string(),
            base: "main".to_string(),
            ..self
        }
    }
}

pub trait GithubPullRequestOpen {
    /// Opens a PR on github, and returns true if it was created successfully.
    fn open(&self, pull_request: &GithubPullRequest) -> impl Future<Output = Result<bool>> + Send;
}

impl GithubPullRequestOpen for Client {
    async fn open(&self, pull_request: &GithubPullRequest) -> Result<bool> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls",
            pull_request.owner, pull_request.repo
        );
        let response = self
            .post(url)
            .bearer_auth(env::var("GITHUB_API_KEY").unwrap())
            .header(CONTENT_TYPE, "application/json")
            .header(USER_AGENT, "roswaal-tif-bot")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .json(&pull_request)
            .send()
            .await?;
        if !response.status().is_success() {
            log::error!("Failed to open PR with status code {}.", response.status());
            return Ok(false);
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        git::branch_name::{self, RoswaalOwnedGitBranchName},
        language::ast::RoswaalTestSyntax,
        location::location::RoswaalStringLocations,
        tests_data::query::RoswaalTestNamesString,
    };

    use super::GithubPullRequest;

    #[test]
    fn test_do_not_merge_specifies_do_not_merge_in_title_and_body() {
        let pr = GithubPullRequest::for_tif_react_frontend(
            "Hello",
            "World",
            &RoswaalOwnedGitBranchName::new("test-branch"),
        )
        .for_testing_do_not_merge();
        assert_eq!(pr.title, "[Test - DO NOT MERGE] Roswaal: Hello");
        assert!(pr
            .body
            .starts_with("This is a test PR, please do not meeeeeeerge!!!\n\n"))
    }

    #[test]
    fn test_from_string_locations_with_invalid_locations() {
        let locations_str = "
Test 1, 45.0, 4.0
908308
Test 2, -78.290782973, 54.309983793
Invalid, hello, world
            ";
        let string_locations = RoswaalStringLocations::from_roswaal_locations_str(locations_str);
        let branch_name = RoswaalOwnedGitBranchName::new("test-locations-branch");
        let pr =
            GithubPullRequest::for_locations_tif_react_frontend(&string_locations, &branch_name);
        assert_eq!(
            pr.title,
            "Roswaal: Add Locations (Test 1, 908308, Test 2, Invalid)".to_string()
        );
        let expected_body = "Adds the following locations to the acceptance teeeeeeeeeests:
- **Test 1** (Latitude: 45.0000000000000000, Longitude: 4.0000000000000000)
- **Test 2** (Latitude: -78.2907867431640625, Longitude: 54.3099822998046875)

The following locations were specified in the slack command, but are invaaaaaaaalid:
- **908308** (Invalid Name)
- **Invalid** (Invalid Coordinate)
";
        assert!(pr.body.contains(expected_body));
    }

    #[test]
    fn test_from_string_locations_only_valid_locations_omits_invalid_section() {
        let locations_str = "
Test 1, 45.0, 4.0
Test 2, -78.290782973, 54.309983793
            ";
        let string_locations = RoswaalStringLocations::from_roswaal_locations_str(locations_str);
        let branch_name = RoswaalOwnedGitBranchName::new("test-locations-branch");
        let pr =
            GithubPullRequest::for_locations_tif_react_frontend(&string_locations, &branch_name);
        assert_eq!(
            pr.title,
            "Roswaal: Add Locations (Test 1, Test 2)".to_string()
        );
        let expected_body = "Adds the following locations to the acceptance teeeeeeeeeests:
- **Test 1** (Latitude: 45.0000000000000000, Longitude: 4.0000000000000000)
- **Test 2** (Latitude: -78.2907867431640625, Longitude: 54.3099822998046875)
";
        assert!(pr.body.contains(expected_body));
    }

    #[test]
    fn test_from_multiple_test_cases() {
        let test1 = "New Test: I am the test
Step 1: Stuff
Step 2: More Stuff
Set Location: Antarctica
Step 3: More Stuff
Requirement 1: Do stuff
Requirement 2: Do more stuff
Requirement 3: Do more stuff";
        let test2 = "New Test: I am the next test
Step 1: A
Requirement 1: B";
        let t1_syntax = RoswaalTestSyntax::from(test1);
        let t2_syntax = RoswaalTestSyntax::from(test2);
        let tests_with_names = vec![
            ("I am the test", &t1_syntax),
            ("I am the next test", &t2_syntax),
        ];
        let pr = GithubPullRequest::for_test_cases_tif_react_frontend(
            &tests_with_names,
            &RoswaalOwnedGitBranchName::new("test-cases"),
        );
        assert_eq!(
            pr.title,
            "Roswaal: Add Tests \"I am the test\", \"I am the next test\""
        );
        let expected_body = "Adds the following teeeeeests!

```
New Test: I am the test
Step 1: Stuff
Step 2: More Stuff
Set Location: Antarctica
Step 3: More Stuff
Requirement 1: Do stuff
Requirement 2: Do more stuff
Requirement 3: Do more stuff
```
```
New Test: I am the next test
Step 1: A
Requirement 1: B
```";
        assert!(pr.body.contains(expected_body))
    }

    #[test]
    fn remove_test_cases() {
        let test_names = RoswaalTestNamesString::new(
            "\
Blob
Blob Jr.
",
        );
        let branch_name = RoswaalOwnedGitBranchName::new("test");
        let pr = GithubPullRequest::for_removing_test_cases_tif_react_frontend(
            &test_names,
            &branch_name,
        );
        assert_eq!(pr.title(), "Roswaal: Remove Tests Blob, Blob Jr.");
        let expected_body = "\
Removes the following teeeeeeeests!
- Blob
- Blob Jr.
";
        assert!(pr.body.contains(expected_body))
    }
}
