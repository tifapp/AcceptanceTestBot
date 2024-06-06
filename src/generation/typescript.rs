use crate::{is_case, language::test::{RoswaalTest, RoswaalTestCommand}, utils::string::{ToAsciiCamelCase, UppercaseFirstAsciiCharacter}};

pub struct GeneratedTypescript {
    test_case_code: String,
    test_action_code: String
}

/// A test command comes from plain-english like scripts describing an
/// acceptance test, and describes how to generate the code for the test.
pub trait RoswaalTypescriptGenerate {
    /// The associated typescript code for this test command.
    fn typescript(&self) -> GeneratedTypescript;
}

impl RoswaalTypescriptGenerate for RoswaalTestCommand {
    fn typescript(&self) -> GeneratedTypescript {
        match self {
            Self::Step { name, requirement } => {
                let function_name = requirement.to_ascii_camel_case();
                GeneratedTypescript {
                    test_case_code: String::new(),
                    test_action_code: format!(
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
            }
            Self::SetLocation { location_name } => {
                GeneratedTypescript {
                    test_case_code: String::new(),
                    test_action_code: format!(
"\
export const setLocationTo{} = async () => {{
  await setUserLocation(TestLocations.{})
}}
",
                        location_name.name().to_ascii_pascal_case(),
                        location_name.name().to_ascii_pascal_case()
                    )
                }
            }
        }
    }
}

/// A test action command that outputs a TestActions.ts file.
pub struct TestActionsCommand {
    commands: Vec<RoswaalTestCommand>
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

impl RoswaalTypescriptGenerate for RoswaalTest {
    fn typescript(&self) -> GeneratedTypescript {
        GeneratedTypescript {
            test_case_code: String::new(),
            test_action_code: self.test_action_typescript()
        }
    }
}

impl RoswaalTest {
    fn test_action_typescript(&self) -> String {
        let mut ts = LAUNCH_IMPORT.to_string();
        let has_location_command = self.commands().iter()
            .find(|c| is_case!(c, RoswaalTestCommand::SetLocation))
            .is_some();
        if has_location_command {
            ts.push_str(LOCATION_IMPORT)
        }
        ts.push_str("\n");
        ts.push_str(BEFORE_LAUNCH_FUNCTION);
        ts.push_str("\n");
        self.commands().iter().enumerate()
            .fold(ts, |mut acc, (i, command)| {
                let suffix = if i < self.commands().len() - 1 { "\n" } else { "" };
                acc.push_str(&(command.typescript().test_action_code + suffix));
                return acc
            })
    }
}

#[cfg(test)]
mod roswaal_command_tests {
    use std::str::FromStr;

    use crate::language::location::RoswaalLocationName;

    use super::*;

    #[test]
    fn test_step_command_action_typescript() {
        let command = RoswaalTestCommand::Step {
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
        assert_eq!(ts.test_action_code, expected_ts.to_string())
    }

    #[test]
    fn test_set_location_command_action_typescript() {
        let command = RoswaalTestCommand::SetLocation {
            location_name: RoswaalLocationName::from_str("San Francisco")
                .unwrap()
        };
        let ts = command.typescript();
        let expected_ts = "\
export const setLocationToSanFrancisco = async () => {
  await setUserLocation(TestLocations.SanFrancisco)
}
";
        assert_eq!(ts.test_action_code, expected_ts.to_string())
    }

    #[test]
    fn test_generate_test_actions_command_typescript_only_steps() {
        let step1 = RoswaalTestCommand::Step {
            name: "Johnny is signed in".to_string(),
            requirement: "Ensure Johnny is signed into his account".to_string()
        };
        let step2 = RoswaalTestCommand::Step {
            name: "Johnny is bored".to_string(),
            requirement: "Ensure that Johnny is not bored".to_string()
        };
        let ts = RoswaalTest::new("A".to_string(), None, vec![step1, step2]).typescript();
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
        assert_eq!(ts.test_action_code, expected_ts.to_string())
    }

    #[test]
    fn test_generate_test_actions_command_typescript_steps_and_location_changes() {
        let command1 = RoswaalTestCommand::Step {
            name: "Johnny is signed in".to_string(),
            requirement: "Ensure Johnny is signed into his account".to_string()
        };
        let command2 = RoswaalTestCommand::SetLocation {
            location_name: RoswaalLocationName::from_str("Oakland").unwrap()
        };
        let ts = RoswaalTest::new("A".to_string(), None, vec![command1, command2]).typescript();
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
        assert_eq!(ts.test_action_code, expected_ts.to_string())
    }
}
