use nanoid::nanoid;
use sqlx::{sqlite::SqliteTypeInfo, Decode, Sqlite, Type};

/// A type for a git branch name that is created by roswaal.
///
/// Each branch name contains a 10 character nano id as its suffix in order to make each instance
/// unique. This uniqueness ensures that duplicate branch names do not clash with each other.
#[derive(Debug, PartialEq, Eq, Clone, Decode)]
pub struct RoswaalOwnedGitBranchName(String);

impl RoswaalOwnedGitBranchName {
    pub fn new(name: &str) -> Self {
        Self(format!("roswaal-{}-{}", name, nanoid!(10)))
    }
}

impl ToString for RoswaalOwnedGitBranchName {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl Type<Sqlite> for RoswaalOwnedGitBranchName {
    fn type_info() -> SqliteTypeInfo {
        <String as Type<Sqlite>>::type_info()
    }
}
