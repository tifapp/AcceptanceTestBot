use super::{ast::RoswaalTestSyntax, test::RoswaalTest};

#[derive(Debug, PartialEq, Eq)]
pub struct RoswaalCompilationError {
    line_number: u32,
    code: RoswaalCompilationErrorCode
}

#[derive(Debug, PartialEq, Eq)]
pub enum RoswaalCompilationErrorCode {
    NoTestName,
    NoTestSteps,
    NoStepDescription { step_name: String },
    NoLocationSpecified,
    InvalidCommandName(String)
}

/// A trait for self-initializing by compiling roswaal test syntax.
pub trait RoswaalCompile: Sized {
    fn compile_syntax(
        syntax: RoswaalTestSyntax
    ) -> Result<Self, RoswaalCompilationError>;

    fn compile(
        source_code: &str
    ) -> Result<Self, RoswaalCompilationError> {
        Self::compile_syntax(RoswaalTestSyntax::from(source_code))
    }
}

impl RoswaalCompile for RoswaalTest {
    fn compile_syntax(syntax: RoswaalTestSyntax) -> Result<Self, RoswaalCompilationError> {
        let mut lines = syntax.source_code().lines();
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
            if step_name.starts_with("Step 1") {
                let error = RoswaalCompilationError {
                    line_number: 2,
                    code: RoswaalCompilationErrorCode::NoStepDescription {
                        step_name
                    }
                };
                return Err(error)
            } else if step_name.starts_with("Set Location") {
                let error = RoswaalCompilationError {
                    line_number: 2,
                    code: RoswaalCompilationErrorCode::NoLocationSpecified
                };
                return Err(error)
            }
            let error = RoswaalCompilationError {
                line_number: 2,
                code: RoswaalCompilationErrorCode::InvalidCommandName(step_name)
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
mod compiler_tests {
    use super::*;

    #[test]
    fn test_parse_returns_no_name_for_empty_string() {
        let result = RoswaalTest::compile("");
        let error = RoswaalCompilationError {
            line_number: 1,
            code: RoswaalCompilationErrorCode::NoTestName
        };
        assert_eq!(result, Err(error))
    }

    #[test]
    fn test_parse_returns_no_name_for_random_string() {
        let result = RoswaalTest::compile("jkashdkjashdkjahsd ehiuh3ui2geuyg23urg");
        let error = RoswaalCompilationError {
            line_number: 1,
            code: RoswaalCompilationErrorCode::NoTestName
        };
        assert_eq!(result, Err(error))
    }

    #[test]
    fn test_parse_returns_no_steps_when_name_formatted_correctly_uppercase() {
        let result = RoswaalTest::compile("New Test: Hello world");
        let error = RoswaalCompilationError {
            line_number: 1,
            code: RoswaalCompilationErrorCode::NoTestSteps
        };
        assert_eq!(result, Err(error))
    }

    #[test]
    fn test_parse_returns_no_steps_when_name_formatted_correctly_lowercase() {
        let result = RoswaalTest::compile("new test: Hello world");
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
        let result = RoswaalTest::compile(test);
        assert_eq!(result, Err(error))
    }

    #[test]
    fn test_parse_returns_invalid_command_name_when_command_is_not_a_step() {
        let test = "\
            new test: Hello world
            passo 1: mamma mia
            ";
        let result = RoswaalTest::compile(test);
        let step_name = String::from("passo 1");
        let error = RoswaalCompilationError {
            line_number: 2,
            code: RoswaalCompilationErrorCode::InvalidCommandName(step_name)
        };
        assert_eq!(result, Err(error))
    }

    #[test]
    fn test_parse_returns_no_step_description_when_step_lacks_description() {
        let test = "\
            New Test: Hello wordl
            Step 1:
            ";
        let result = RoswaalTest::compile(test);
        let error = RoswaalCompilationError {
            line_number: 2,
            code: RoswaalCompilationErrorCode::NoStepDescription {
                step_name: "Step 1".to_string()
            }
        };
        assert_eq!(result, Err(error))
    }

    #[test]
    fn test_parse_returns_no_location_specified_when_location_step_is_empty() {
        let test = "\
            New test: This is an acceptance test
            Set Location:
            ";
        let result = RoswaalTest::compile(test);
        let error = RoswaalCompilationError {
            line_number: 2,
            code: RoswaalCompilationErrorCode::NoLocationSpecified
        };
        assert_eq!(result, Err(error))
    }
}
