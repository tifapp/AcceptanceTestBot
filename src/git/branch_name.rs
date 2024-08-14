use nanoid::nanoid;
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};
use serde::{
    de::{Unexpected, Visitor},
    Deserialize, Serialize,
};
use sqlx::{sqlite::SqliteTypeInfo, Decode, Encode, Sqlite, Type};

/// A type for a git branch name that is created by roswaal.
///
/// Each branch name contains a 10 character nano id as its suffix in order to make each instance
/// unique. This uniqueness ensures that duplicate branch names do not clash with each other.
#[derive(Debug, PartialEq, Eq, Clone, Decode, Encode, Serialize)]
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
}

/// A specific type of branch created by this tool.
#[derive(Debug, PartialEq, Eq)]
pub enum RoswaalOwnedBranchKind {
    AddLocations,
    AddTests,
    RemoveTests,
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

    /// Returns the kind of branch that this name represents, or none if it does not represent a
    /// production purpose of this tool.
    pub fn kind(&self) -> Option<RoswaalOwnedBranchKind> {
        if self.is_named("add-tests") {
            Some(RoswaalOwnedBranchKind::AddTests)
        } else if self.is_named("add-locations") {
            Some(RoswaalOwnedBranchKind::AddLocations)
        } else if self.is_named("remove-tests") {
            Some(RoswaalOwnedBranchKind::RemoveTests)
        } else {
            None
        }
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

impl<'d> Deserialize<'d> for RoswaalOwnedGitBranchName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'d>,
    {
        deserializer.deserialize_str(RoswaalGitBranchNameVisitor)
    }
}

struct RoswaalGitBranchNameVisitor;

static BRANCH_NAME_REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"^roswaal-.*-.{10}$")
        .build()
        .expect("Failed to compile location name regex.")
});

impl<'de> Visitor<'de> for RoswaalGitBranchNameVisitor {
    type Value = RoswaalOwnedGitBranchName;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("The format roswaal-<name>-<nano_id>.")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if BRANCH_NAME_REGEX.is_match(v) {
            Ok(RoswaalOwnedGitBranchName(v.to_string()))
        } else {
            Err(serde::de::Error::invalid_value(Unexpected::Str(v), &self))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::git::branch_name::RoswaalOwnedBranchKind;

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
        assert!(!branch_name.is_named(&branch_name.to_string()));
        assert!(branch_name.is_named("test-branch"));
    }

    #[test]
    fn test_kind() {
        let names_to_kind = vec![
            (RoswaalOwnedGitBranchName::new("test-branch"), None),
            (
                RoswaalOwnedGitBranchName::for_adding_tests(),
                Some(RoswaalOwnedBranchKind::AddTests),
            ),
            (
                RoswaalOwnedGitBranchName::for_removing_tests(),
                Some(RoswaalOwnedBranchKind::RemoveTests),
            ),
            (
                RoswaalOwnedGitBranchName::for_adding_locations(),
                Some(RoswaalOwnedBranchKind::AddLocations),
            ),
            (RoswaalOwnedGitBranchName::new("i-am-groot"), None),
        ];
        for (name, kind) in names_to_kind {
            assert_eq!(name.kind(), kind);
        }
    }

    #[test]
    fn deserialize_invalid_branches() {
        let branches = vec![
            "\"slkhjndkljshjdfjkshdfkj\"",
            "\"roswaal-hjskjsh\"",
            "\"roswaal-add-tests-ksdjhbnkjbdjkhbdjhbdjbjdhbjdb\"",
        ];
        for branch in branches {
            assert!(serde_json::from_str::<RoswaalOwnedGitBranchName>(branch).is_err())
        }
    }

    #[test]
    fn deserialize_valid_branch() {
        assert!(serde_json::from_str::<RoswaalOwnedGitBranchName>(
            "\"roswaal-remove-tests-G983j839s4\""
        )
        .is_ok())
    }
}
