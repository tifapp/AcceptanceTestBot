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

    pub fn for_adding_tests() -> Self {
        Self::new("add-tests")
    }

    pub fn for_removing_tests() -> Self {
        Self::new("remove-tests")
    }

    pub fn for_adding_locations() -> Self {
        Self::new("add-locations")
    }

    pub fn for_removing_locations() -> Self {
        Self::new("remove-locations")
    }
}

impl RoswaalOwnedGitBranchName {
    /// Returns true if the branch's base name is the specifed name.
    ///
    /// This check does not include any special characters added by this type that can be found
    /// in the `.to_string()` output.
    pub fn is_named(&self, name: &str) -> bool {
        let (start, end) = (8, self.0.len() - 11);
        &self.0[start..end] == name
    }

    pub fn is_for_adding_tests(&self) -> bool {
        self.is_named("add-tests")
    }

    pub fn is_for_removing_tests(&self) -> bool {
        self.is_named("remove-tests")
    }

    pub fn is_for_adding_locations(&self) -> bool {
        self.is_named("add-locations")
    }

    pub fn is_for_removing_locations(&self) -> bool {
        self.is_named("remove-locations")
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

#[cfg(test)]
mod tests {
    use super::RoswaalOwnedGitBranchName;

    #[test]
    fn test_is_named() {
        let branch_name = RoswaalOwnedGitBranchName::new("test-branch");
        let nanoid = &branch_name.to_string()[branch_name.to_string().len() - 10..];
        assert!(!branch_name.is_named("roswaal"));
        assert!(!branch_name.is_named(nanoid));
        assert!(!branch_name.is_named("-"));
        assert!(!branch_name.is_named("test"));
        assert!(!branch_name.is_named("branch"));
        assert!(branch_name.is_named("test-branch"));
    }
}
