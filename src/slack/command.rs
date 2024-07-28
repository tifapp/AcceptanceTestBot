use std::str::FromStr;
use serde::{de::{Unexpected, Visitor}, Deserialize};
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, EnumString, IntoStaticStr};

/// The slack slash commands that this tool must respond to.
#[derive(Debug, PartialEq, Eq, EnumString, EnumIter, IntoStaticStr, Clone, Copy)]
pub enum RoswaalSlackCommand {
    #[strum(serialize="/view-tests")]
    ViewTests,
    #[strum(serialize="/add-tests")]
    AddTests,
    #[strum(serialize="/remove-tests")]
    RemoveTests,
    #[strum(serialize="/view-locations")]
    ViewLocations,
    #[strum(serialize="/add-locations")]
    AddLocations
}

impl RoswaalSlackCommand {
    /// Returns true if on average this command runs longer than the 3 seconds that slack allows
    /// for responding to commands.
    ///
    /// Any commands that would imply an interaction with git, github, or an external service
    /// should return true from this method.
    pub fn is_long_running(&self) -> bool {
        match self {
            Self::AddTests | Self::AddLocations | Self::RemoveTests => true,
            _ => false
        }
    }
}

impl <'d> Deserialize<'d> for RoswaalSlackCommand {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'d> {
        deserializer.deserialize_str(RoswaalSlackCommandVistor)
    }
}

struct RoswaalSlackCommandVistor;

impl <'de> Visitor<'de> for RoswaalSlackCommandVistor {
    type Value = RoswaalSlackCommand;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        let commands = RoswaalSlackCommand::iter().map(|c| c.into()).collect::<Vec<&str>>();
        formatter.write_str(&format!("Any of {}.", commands.join(", ")))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: serde::de::Error {
        RoswaalSlackCommand::from_str(v)
            .map_err(|_| serde::de::Error::invalid_value(Unexpected::Str(v), &self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq, Deserialize)]
    struct Container {
        command: RoswaalSlackCommand
    }

    #[test]
    fn deserialize() {
        let json = r#"{"command": "/view-tests"}"#;
        let container = serde_json::from_str::<Container>(json).unwrap();
        assert_eq!(container, Container { command: RoswaalSlackCommand::ViewTests })
    }
}
