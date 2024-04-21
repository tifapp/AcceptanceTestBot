use crate::{is_case, utils::string::{ToAsciiCamelCase, UppercaseFirstAsciiCharacter}};

use super::location::RoswaalLocationName;

/// A test command comes from plain-english like scripts describing an
/// acceptance test, and describes how to generate the code for the test.
pub trait RoswaalTestCommand {
    /// The associated typescript code for this test command.
    fn typescript(&self) -> String;
}

/// A command which represents a function in a TestActions.ts file.
pub enum TestActionFunctionCommand {
    Step { name: String, requirement: String },
    SetLocation { location_name: RoswaalLocationName }
}

impl RoswaalTestCommand for TestActionFunctionCommand {
    fn typescript(&self) -> String {
        match self {
            Self::Step { name, requirement } => {
                let function_name = requirement.to_ascii_camel_case();
                format!(
"\
export const {} = async () => {{
  // {}
  throw new Error(\"TODO\")
}}
",
                    function_name,
                    name
                )
            }
            Self::SetLocation { location_name } => {
                format!(
"\
export const setLocationTo{} = async () => {{
  await setUserLocation(TestLocations.{})
}}
",
                    location_name.raw_value.to_ascii_pascal_case(),
                    location_name.raw_value.to_ascii_pascal_case()
                )
            }
        }
    }
}

/// A test action command that outputs a TestActions.ts file.
pub struct TestActionsCommand {
    commands: Vec<TestActionFunctionCommand>
}

const LAUNCH_IMPORT: &str = "import { TestAppLaunchConfig } from \"../Launch\"\n";
const LOCATION_IMPORT: &str = "import { TestLocations, setUserLocation } from \"../Location\"\n";
const BEFORE_LAUNCH_FUNCTION: &str = "\
export const beforeLaunch = async (): Promise<TestAppLaunchConfig> => {
  // Perform any setup work in here, (setting location, reseting device
  // permissions, etc.)
  return {}
}
";

impl RoswaalTestCommand for TestActionsCommand {
    fn typescript(&self) -> String {
        let mut ts = LAUNCH_IMPORT.to_string();
        let has_location_command = self.commands.iter()
            .find(|c| is_case!(c, TestActionFunctionCommand::SetLocation))
            .is_some();
        if has_location_command {
            ts.push_str(LOCATION_IMPORT)
        }
        ts.push_str("\n");
        ts.push_str(BEFORE_LAUNCH_FUNCTION);
        ts.push_str("\n");
        return self.commands.iter().enumerate()
            .fold(ts, |mut acc, (i, command)| {
                let suffix = if i < self.commands.len() - 1 { "\n" } else { "" };
                acc.push_str(&(command.typescript() + suffix));
                return acc
            })
    }
}

#[cfg(test)]
mod roswaal_command_tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_step_command_typescript() {
        let command = TestActionFunctionCommand::Step {
            name: String::from("Anna is about to arrive at an event"),
            requirement: String::from("Mark Anna as being present at an event")
        };
        let ts = command.typescript();
        let expected_ts = "\
export const markAnnaAsBeingPresentAtAnEvent = async () => {
  // Anna is about to arrive at an event
  throw new Error(\"TODO\")
}
";
        assert_eq!(ts, String::from(expected_ts))
    }

    #[test]
    fn test_set_location_command_typescript() {
        let command = TestActionFunctionCommand::SetLocation {
            location_name: RoswaalLocationName::from_str("San Francisco")
                .unwrap()
        };
        let ts = command.typescript();
        let expected_ts = "\
export const setLocationToSanFrancisco = async () => {
  await setUserLocation(TestLocations.SanFrancisco)
}
";
        assert_eq!(ts, String::from(expected_ts))
    }

    #[test]
    fn test_generate_test_actions_command_typescript_only_steps() {
        let step1 = TestActionFunctionCommand::Step {
            name: "Johnny is signed in".to_string(),
            requirement: "Ensure Johnny is signed into his account".to_string()
        };
        let step2 = TestActionFunctionCommand::Step {
            name: "Johnny is bored".to_string(),
            requirement: "Ensure that Johnny is not bored".to_string()
        };
        let command = TestActionsCommand {
            commands: vec!(step1, step2)
        };
        let ts = command.typescript();
        let expected_ts = "\
import { TestAppLaunchConfig } from \"../Launch\"

export const beforeLaunch = async (): Promise<TestAppLaunchConfig> => {
  // Perform any setup work in here, (setting location, reseting device
  // permissions, etc.)
  return {}
}

export const ensureJohnnyIsSignedIntoHisAccount = async () => {
  // Johnny is signed in
  throw new Error(\"TODO\")
}

export const ensureThatJohnnyIsNotBored = async () => {
  // Johnny is bored
  throw new Error(\"TODO\")
}
";
        assert_eq!(ts, expected_ts.to_string())
    }

    #[test]
    fn test_generate_test_actions_command_typescript_steps_and_location_changes() {
        let command1 = TestActionFunctionCommand::Step {
            name: "Johnny is signed in".to_string(),
            requirement: "Ensure Johnny is signed into his account".to_string()
        };
        let command2 = TestActionFunctionCommand::SetLocation {
            location_name: RoswaalLocationName::from_str("Oakland").unwrap()
        };
        let command = TestActionsCommand {
            commands: vec!(command1, command2)
        };
        let ts = command.typescript();
        let expected_ts = "\
import { TestAppLaunchConfig } from \"../Launch\"
import { TestLocations, setUserLocation } from \"../Location\"

export const beforeLaunch = async (): Promise<TestAppLaunchConfig> => {
  // Perform any setup work in here, (setting location, reseting device
  // permissions, etc.)
  return {}
}

export const ensureJohnnyIsSignedIntoHisAccount = async () => {
  // Johnny is signed in
  throw new Error(\"TODO\")
}

export const setLocationToOakland = async () => {
  await setUserLocation(TestLocations.Oakland)
}
";
        assert_eq!(ts, expected_ts.to_string())
    }
}
