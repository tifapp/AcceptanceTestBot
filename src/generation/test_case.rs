use anyhow::Result;
use tokio::{
    fs::{create_dir_all, File},
    io::AsyncWriteExt,
};

use crate::{
    is_case,
    language::test::{RoswaalCompiledTest, RoswaalCompiledTestCommand},
    utils::string::ToAsciiCamelCase,
};

use super::{constants::GENERATED_HEADER, interface::RoswaalTypescriptGenerate};

/// An output of generating typescript code.
pub struct TestCaseTypescript {
    test_case_code: String,
    test_action_code: String,
}

impl TestCaseTypescript {
    /// Saves this typescript code in files in the specified dirpath.
    pub async fn save_in_dir(&self, dirpath: &str) -> Result<()> {
        create_dir_all(dirpath).await?;
        let mut file = File::create(format!("{}/TestCase.test.ts", dirpath)).await?;
        file.write(self.test_case_code.as_bytes()).await?;
        file = File::create(format!("{}/TestActions.ts", dirpath)).await?;
        file.write(self.test_action_code.as_bytes()).await?;
        Ok(())
    }
}

impl RoswaalTypescriptGenerate<TestCaseTypescript> for RoswaalCompiledTestCommand {
    fn typescript(&self) -> TestCaseTypescript {
        match self {
            Self::Step {
                label: _,
                name,
                requirement,
            } => {
                let mut function_name = requirement.to_ascii_camel_case();
                function_name.retain(|c| !r#"()$@#*,".;:'!"#.contains(c));
                TestCaseTypescript {
                    test_case_code: format!(
                        "\
  // {}
  testCase.appendAction(TestActions.{})
",
                        name, function_name
                    ),
                    test_action_code: format!(
                        "\
export const {} = async () => {{
  // {}
  throw new Error(\"TODO\")
}}
",
                        function_name, name
                    ),
                }
            }
            Self::SetLocation { location_name } => {
                let function_name = format!(
                    "setLocationTo{}",
                    location_name.to_ascii_pascal_case_string()
                );
                TestCaseTypescript {
                    test_case_code: format!(
                        "\
  // Set Location to {}
  testCase.appendAction(TestActions.{})
",
                        location_name.raw_name(),
                        function_name
                    ),
                    test_action_code: format!(
                        "\
export const {} = async () => {{
  await setUserLocation(TestLocations.{})
}}
",
                        function_name,
                        location_name.to_ascii_pascal_case_string()
                    ),
                }
            }
        }
    }
}

const TEST_ACTIONS_LAUNCH_IMPORT: &str = "import { TestAppLaunchConfig } from \"../Launch\"\n";
const TEST_ACTIONS_LOCATION_IMPORT: &str =
    "import { TestLocations, setUserLocation } from \"../Location\"\n";
const TEST_ACTIONS_BEFORE_LAUNCH_FUNCTION: &str = "\
export const beforeLaunch = async (): Promise<TestAppLaunchConfig> => {
  // Perform any setup work in here, (setting location, reseting device
  // permissions, etc.)
  return {}
}
";
const TEST_CASE_IMPORTS: &str = "\
import * as TestActions from \"./TestActions\"
import { launchApp } from \"../Launch\"
import { RoswaalTestCase } from \"../TestCase\"
import { roswaalClient } from \"../Client\"

";
const TEST_CASE_END: &str = "\
await roswaalClient.run(testCase)
})
";
const TEST_CASE_APPEND_ACTION_SPACING: &str = "  ";

fn test_case_test_block_start(name: &str) -> String {
    let escaped_name = name.replace(r#"""#, r#"\""#);
    format!(
        "\
test(\"{}\", async () => {{
  const testCase = new RoswaalTestCase(\"{}\", TestActions.beforeLaunch)
",
        escaped_name, escaped_name
    )
}

impl RoswaalTypescriptGenerate<TestCaseTypescript> for RoswaalCompiledTest {
    fn typescript(&self) -> TestCaseTypescript {
        TestCaseTypescript {
            test_case_code: self.test_case_typescript(),
            test_action_code: self.test_action_typescript(),
        }
    }
}

impl RoswaalCompiledTest {
    fn test_case_typescript(&self) -> String {
        let mut ts = GENERATED_HEADER.to_string();
        ts.push_str(TEST_CASE_IMPORTS);
        ts.push_str(&test_case_test_block_start(self.name()));
        ts.push_str(TEST_CASE_APPEND_ACTION_SPACING);
        for code in self
            .commands()
            .iter()
            .map(|c| c.typescript().test_case_code)
        {
            ts.push_str(&code);
            ts.push_str(TEST_CASE_APPEND_ACTION_SPACING);
        }
        ts.push_str(TEST_CASE_END);
        ts
    }

    fn test_action_typescript(&self) -> String {
        let mut ts = TEST_ACTIONS_LAUNCH_IMPORT.to_string();
        let has_location_command = self
            .commands()
            .iter()
            .find(|c| is_case!(c, RoswaalCompiledTestCommand::SetLocation))
            .is_some();
        if has_location_command {
            ts.push_str(TEST_ACTIONS_LOCATION_IMPORT)
        }
        ts.push_str("\n");
        ts.push_str(TEST_ACTIONS_BEFORE_LAUNCH_FUNCTION);
        ts.push_str("\n");
        self.commands()
            .iter()
            .enumerate()
            .fold(ts, |mut acc, (i, command)| {
                let suffix = if i < self.commands().len() - 1 {
                    "\n"
                } else {
                    ""
                };
                acc.push_str(&(command.typescript().test_action_code + suffix));
                return acc;
            })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::location::name::RoswaalLocationName;

    use super::*;

    #[test]
    fn test_step_command_action_typescript() {
        let command = RoswaalCompiledTestCommand::Step {
            label: "Step 1".to_string(),
            name: String::from("Anna is about to arrive at an event"),
            requirement: String::from("Mark Anna as being present at an event"),
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
    fn test_step_command_test_case_typescript() {
        let command = RoswaalCompiledTestCommand::Step {
            label: "Step 1".to_string(),
            name: String::from("Anna is about to arrive at an event"),
            requirement: String::from("Mark Anna as being present at an event"),
        };
        let ts = command.typescript();
        let expected_ts = "\
  // Anna is about to arrive at an event
  testCase.appendAction(TestActions.markAnnaAsBeingPresentAtAnEvent)
";
        assert_eq!(ts.test_case_code, expected_ts.to_string())
    }

    #[test]
    fn test_set_location_command_action_typescript() {
        let command = RoswaalCompiledTestCommand::SetLocation {
            location_name: RoswaalLocationName::from_str("San Francisco").unwrap(),
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
    fn test_set_location_command_test_case_typescript() {
        let command = RoswaalCompiledTestCommand::SetLocation {
            location_name: RoswaalLocationName::from_str("San Francisco").unwrap(),
        };
        let ts = command.typescript();
        let expected_ts = "\
  // Set Location to San Francisco
  testCase.appendAction(TestActions.setLocationToSanFrancisco)
";
        assert_eq!(ts.test_case_code, expected_ts.to_string())
    }

    #[test]
    fn test_generate_test_actions_command_typescript_only_steps() {
        let step1 = RoswaalCompiledTestCommand::Step {
            label: "Step 1".to_string(),
            name: "Johnny is signed in".to_string(),
            requirement: "Ensure Johnny is signed into his account".to_string(),
        };
        let step2 = RoswaalCompiledTestCommand::Step {
            label: "Step 2".to_string(),
            name: "Johnny is bored".to_string(),
            requirement: "Ensure that Johnny is not bored".to_string(),
        };
        let ts = RoswaalCompiledTest::new("A".to_string(), None, vec![step1, step2]).typescript();
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
        let command1 = RoswaalCompiledTestCommand::Step {
            label: "Step 1".to_string(),
            name: "Johnny is signed in".to_string(),
            requirement: "Ensure Johnny is signed into his account,,,, and is (*$)(*)($# alive"
                .to_string(),
        };
        let command2 = RoswaalCompiledTestCommand::SetLocation {
            location_name: RoswaalLocationName::from_str("Oakland").unwrap(),
        };
        let ts =
            RoswaalCompiledTest::new("A".to_string(), None, vec![command1, command2]).typescript();
        let expected_ts = "\
import { TestAppLaunchConfig } from \"../Launch\"
import { TestLocations, setUserLocation } from \"../Location\"

export const beforeLaunch = async (): Promise<TestAppLaunchConfig> => {
  // Perform any setup work in here, (setting location, reseting device
  // permissions, etc.)
  return {}
}

export const ensureJohnnyIsSignedIntoHisAccountAndIsAlive = async () => {
  // Johnny is signed in
  throw new Error(\"TODO\")
}

export const setLocationToOakland = async () => {
  await setUserLocation(TestLocations.Oakland)
}
";
        assert_eq!(ts.test_action_code, expected_ts.to_string())
    }

    #[test]
    fn test_generate_test_case_command_typescript_only_steps() {
        let step1 = RoswaalCompiledTestCommand::Step {
            label: "Step 1".to_string(),
            name: "Johnny is signed in".to_string(),
            requirement: "Ensure Johnny is signed into his account,,,, and is !!!!(*$)(*)($# alive"
                .to_string(),
        };
        let step2 = RoswaalCompiledTestCommand::Step {
            label: "Step 2".to_string(),
            name: "Johnny is bored".to_string(),
            requirement: "Ensure that Johnny is not bored".to_string(),
        };
        let ts = RoswaalCompiledTest::new("B".to_string(), None, vec![step1, step2]).typescript();
        let expected_ts = "\
// Generated by Roswaal, do not touch.

import * as TestActions from \"./TestActions\"
import { launchApp } from \"../Launch\"
import { RoswaalTestCase } from \"../TestCase\"
import { roswaalClient } from \"../Client\"

test(\"B\", async () => {
  const testCase = new RoswaalTestCase(\"B\", TestActions.beforeLaunch)
  // Johnny is signed in
  testCase.appendAction(TestActions.ensureJohnnyIsSignedIntoHisAccountAndIsAlive)
  // Johnny is bored
  testCase.appendAction(TestActions.ensureThatJohnnyIsNotBored)
  await roswaalClient.run(testCase)
})
";
        assert_eq!(ts.test_case_code, expected_ts.to_string())
    }

    #[test]
    fn test_generate_test_case_command_typescript_steps_and_location_changes() {
        let command1 = RoswaalCompiledTestCommand::Step {
            label: "Step 1".to_string(),
            name: "Johnny is signed in".to_string(),
            requirement: "Ensure Johnny is signed into his account".to_string(),
        };
        let command2 = RoswaalCompiledTestCommand::SetLocation {
            location_name: RoswaalLocationName::from_str("Oakland").unwrap(),
        };
        let ts =
            RoswaalCompiledTest::new("I am \"Bob\"".to_string(), None, vec![command1, command2])
                .typescript();
        let expected_ts = r#"// Generated by Roswaal, do not touch.

import * as TestActions from "./TestActions"
import { launchApp } from "../Launch"
import { RoswaalTestCase } from "../TestCase"
import { roswaalClient } from "../Client"

test("I am \"Bob\"", async () => {
  const testCase = new RoswaalTestCase("I am \"Bob\"", TestActions.beforeLaunch)
  // Johnny is signed in
  testCase.appendAction(TestActions.ensureJohnnyIsSignedIntoHisAccount)
  // Set Location to Oakland
  testCase.appendAction(TestActions.setLocationToOakland)
  await roswaalClient.run(testCase)
})
"#;
        assert_eq!(ts.test_case_code, expected_ts.to_string())
    }
}
