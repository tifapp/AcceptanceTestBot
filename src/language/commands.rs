use crate::utils::string::{ToAsciiCamelCase, UppercaseFirstAsciiCharacter};

use super::location::RoswaalLocationName;

pub trait RoswaalTestCommand {
    fn typescript(&self) -> String;
}

pub struct StepCommand {
    name: String,
    requirement: String
}

impl RoswaalTestCommand for StepCommand {
    fn typescript(&self) -> String {
        let function_name = self.requirement.to_ascii_camel_case();
        format!(
            "\
            export const {} = async () => {{
              // {}
              throw new Error(\"TODO\")
            }}
            ",
            function_name,
            self.name
        )
    }
}

pub struct SetLocationCommand {
    location_name: RoswaalLocationName
}

impl RoswaalTestCommand for SetLocationCommand {
    fn typescript(&self) -> String {
        format!(
            "\
            export const setLocationTo{} = async () => {{
              await setUserLocation(TestLocations.{})
            }}
            ",
            self.location_name.raw_value.to_ascii_pascal_case(),
            self.location_name.raw_value.to_ascii_pascal_case()
        )
    }
}

#[cfg(test)]
mod roswaal_command_tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_step_command_typescript() {
        let command = StepCommand {
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
        let command = SetLocationCommand {
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
}
