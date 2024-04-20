use crate::utils::string::ToAsciiCamelCase;

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

#[cfg(test)]
mod roswaal_command_tests {
    use super::*;

    #[test]
    fn test_step_command_basic_typescript() {
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
}
