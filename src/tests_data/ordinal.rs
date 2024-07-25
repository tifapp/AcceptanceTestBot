use serde::Deserialize;
use sqlx::{prelude::Type, sqlite::SqliteTypeInfo, Decode, Encode, Sqlite};

/// An ordinal that represents the index of a step in a test case.
///
/// Each test case has a before launch step which gets the special zero ordinal that can be
/// obtained through the `for_before_launch` constructor.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Deserialize, Encode, Decode, PartialOrd, Ord)]
pub struct RoswaalTestCommandOrdinal(i32);

impl RoswaalTestCommandOrdinal {
    /// The ordinal for a test's before launch command.
    pub fn for_before_launch() -> Self {
        Self(0)
    }

    /// The ordinal for the index of a command in the list of test commands.
    ///
    /// The index should correspond directly to the position of the command in a vector of commands
    /// that does not include the before launch command.
    pub fn new(command_index: i32) -> Self {
        Self(command_index + 1)
    }
}

impl Type<Sqlite> for RoswaalTestCommandOrdinal {
    fn type_info() -> SqliteTypeInfo {
        <i32 as Type<Sqlite>>::type_info()
    }
}
