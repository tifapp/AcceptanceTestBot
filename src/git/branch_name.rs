use nanoid::nanoid;

/// A type for a git branch name that is created by roswaal.
///
/// Each branch name contains a 10 character nano id as its suffix in order to make each instance
/// unique. This uniqueness ensures that duplicate branch names do not clash with each other.
#[derive(Debug, PartialEq, Eq)]
pub struct RoswaalGitBranchName {
    raw_name: String
}

impl RoswaalGitBranchName {
    pub fn new(name: &str) -> Self {
        Self { raw_name: format!("roswaal:{}:{}", name, nanoid!(10)) }
    }
}

impl ToString for RoswaalGitBranchName {
    fn to_string(&self) -> String {
        self.raw_name.clone()
    }
}
