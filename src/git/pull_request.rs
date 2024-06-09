pub struct GithubPullRequest {
    title: String,
    description: String,
    owner: String,
    repo: String,
    head_branch: String,
    base_branch: String
}

impl GithubPullRequest {
    pub fn for_tif_react_frontend(title: String, description: String, head_branch: String) -> Self {
        Self {
            title,
            description,
            head_branch,
            owner: "tifapp".to_string(),
            repo: "FitnessProject".to_string(),
            base_branch: "development".to_string(),
        }
    }
}
