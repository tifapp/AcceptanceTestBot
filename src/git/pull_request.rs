use std::env;

use anyhow::Result;
use reqwest::{header::CONTENT_TYPE, Client};
use serde::Serialize;

use crate::location::location::{RoswaalLocationStringError, RoswaalStringLocations};

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct GithubPullRequest {
    title: String,
    body: String,
    #[serde(skip)]
    owner: String,
    #[serde(skip)]
    repo: String,
    head: String,
    base: String
}

impl GithubPullRequest {
    /// Creates a PR for the main frontend repo.
    pub fn for_tif_react_frontend(title: String, body: String, head_branch: String) -> Self {
        Self {
            body,
            title: format!("Roswaal: {}", title),
            owner: "tifapp".to_string(),
            repo: "FitnessProject".to_string(),
            base: "development".to_string(),
            head: format!("roswaal:{}", head_branch)
        }
    }

    /// Creates a PR associated with adding new locations to the main frontend repo.
    pub fn for_locations_tif_react_frontend(
        string_locations: RoswaalStringLocations,
        head_branch: String
    ) -> Self {
        let title = format!("Add Locations ({})", string_locations.raw_names().join(", "));
        let mut body = "Adds the following locations to the acceptance teeeeeeeeeests:\n".to_string();
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
                    },
                    RoswaalLocationStringError::InvalidCoordinate { name: _ } => {
                        body.push_str("(Invalid Coordinate)")
                    }
                };
                body.push_str("\n")
            }
        }
        Self::for_tif_react_frontend(title, body, head_branch)
    }
}

pub trait GithubPullRequestOpen {
    /// Opens a PR on github, and returns true if it was created successfully.
    async fn open(&self, pull_request: &GithubPullRequest) -> Result<bool>;
}

impl GithubPullRequestOpen for Client {
    async fn open(&self, pull_request: &GithubPullRequest) -> Result<bool> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls",
            pull_request.owner,
            pull_request.repo
        );
        let response = self.post(url)
            .bearer_auth(env::var("GITHUB_API_KEY").unwrap())
            .header(CONTENT_TYPE, "application/json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .json(&pull_request)
            .send()
            .await?;
        Ok(response.status() == 201)
    }
}

#[cfg(test)]
mod tests {
    use crate::location::location::RoswaalStringLocations;

    use super::GithubPullRequest;

    #[test]
    fn test_from_string_locations_with_invalid_locations() {
        let locations_str = "
Test 1, 45.0, 4.0
908308
Test 2, -78.290782973, 54.309983793
Invalid, hello, world
            ";
        let string_locations = RoswaalStringLocations::from_roswaal_locations_str(locations_str);
        let branch_name = "test-locations-branch".to_string();
        let pr = GithubPullRequest::for_locations_tif_react_frontend(string_locations, branch_name);
        assert_eq!(pr.title, "Roswaal: Add Locations (Test 1, 908308, Test 2, Invalid)".to_string());
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
        let branch_name = "test-locations-branch".to_string();
        let pr = GithubPullRequest::for_locations_tif_react_frontend(string_locations, branch_name);
        assert_eq!(pr.title, "Roswaal: Add Locations (Test 1, Test 2)".to_string());
        let expected_body = "Adds the following locations to the acceptance teeeeeeeeeests:
- **Test 1** (Latitude: 45.0000000000000000, Longitude: 4.0000000000000000)
- **Test 2** (Latitude: -78.2907867431640625, Longitude: 54.3099822998046875)
";
        assert!(pr.body.contains(expected_body));
    }
}
