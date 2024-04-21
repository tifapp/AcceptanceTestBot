use super::compiler::{RoswaalCompilationError, RoswaalCompilationErrorCode};

#[derive(Debug, PartialEq, Eq)]
pub struct RoswaalTest;

impl RoswaalTest {
    pub fn parse(roswaal_string: &String) -> Result<Self, RoswaalCompilationError> {
        let mut lines = roswaal_string.lines();
        let has_test_name = lines.next()
            .map(|l| l.to_lowercase().starts_with("new test:"))
            .unwrap_or(false);
        if !has_test_name {
            let error = RoswaalCompilationError {
                line_number: 1,
                code: RoswaalCompilationErrorCode::NoTestName
            };
            return Err(error);
        }
        let step_line = lines.next();
        if let Some((step_name, _)) = step_line.and_then(|l| l.split_once(":")) {
            let step_name = String::from(step_name.trim());
            let error = RoswaalCompilationError {
                line_number: 2,
                code: RoswaalCompilationErrorCode::InvalidStepName(step_name)
            };
            return Err(error)
        }
        let error = RoswaalCompilationError {
            line_number: if step_line.is_some() { 2 } else { 1 },
            code: RoswaalCompilationErrorCode::NoTestSteps
        };
        Err(error)
    }
}

#[cfg(test)]
mod roswaal_test_tests {
    use crate::language::compiler::RoswaalCompilationError;

    use super::*;

    #[test]
    fn test_parse_returns_no_name_for_empty_string() {
        let result = RoswaalTest::parse(&String::from(""));
        let error = RoswaalCompilationError {
            line_number: 1,
            code: RoswaalCompilationErrorCode::NoTestName
        };
        assert_eq!(result, Err(error))
    }

    #[test]
    fn test_parse_returns_no_name_for_random_string() {
        let test_string = &String::from("jkashdkjashdkjahsd ehiuh3ui2geuyg23urg");
        let result = RoswaalTest::parse(test_string);
        let error = RoswaalCompilationError {
            line_number: 1,
            code: RoswaalCompilationErrorCode::NoTestName
        };
        assert_eq!(result, Err(error))
    }

    #[test]
    fn test_parse_returns_no_steps_when_name_formatted_correctly_uppercase() {
        let test = "New Test: Hello world";
        let result = RoswaalTest::parse(&String::from(test));
        let error = RoswaalCompilationError {
            line_number: 1,
            code: RoswaalCompilationErrorCode::NoTestSteps
        };
        assert_eq!(result, Err(error))
    }

    #[test]
    fn test_parse_returns_no_steps_when_name_formatted_correctly_lowercase() {
        let test = "new test: Hello world";
        let result = RoswaalTest::parse(&String::from(test));
        let error = RoswaalCompilationError {
            line_number: 1,
            code: RoswaalCompilationErrorCode::NoTestSteps
        };
        assert_eq!(result, Err(error))
    }

    #[test]
    fn test_parse_returns_no_steps_when_step_line_is_random_string() {
        let test = "\
            new test: Hello world
            lsjkhadjkhasdfjkhasdjkfhkjsd
            ";
        let error = RoswaalCompilationError {
            line_number: 2,
            code: RoswaalCompilationErrorCode::NoTestSteps
        };
        let result = RoswaalTest::parse(&String::from(test));
        assert_eq!(result, Err(error))
    }

    #[test]
    fn test_parse_returns_invalid_step_name_when_command_is_not_a_step() {
        let test = "\
            new test: Hello world
            passo 1: mamma mia
            ";
        let result = RoswaalTest::parse(&String::from(test));
        let step_name = String::from("passo 1");
        let error = RoswaalCompilationError {
            line_number: 2,
            code: RoswaalCompilationErrorCode::InvalidStepName(step_name)
        };
        assert_eq!(result, Err(error))
    }
}
