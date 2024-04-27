use super::{ast::{RoswaalTestSyntax, RoswaalTestSyntaxToken}, location::RoswaalLocationName, test::RoswaalTest};

#[derive(Debug, PartialEq, Eq)]
pub struct RoswaalCompilationError {
    line_number: u32,
    code: RoswaalCompilationErrorCode
}

#[derive(Debug, PartialEq, Eq)]
pub enum RoswaalCompilationErrorCode {
    NoTestName,
    NoTestSteps,
    NoCommandDescription { command_name: String },
    InvalidLocationName(String),
    InvalidCommandName(String),
    DuplicateTestName(String),
    TestNameAlreadyDeclared
}

pub struct RoswaalCompileContext {
    location_names: Vec<RoswaalLocationName>,
    test_names: Vec<String>
}

impl RoswaalCompileContext {
    pub fn empty() -> Self {
        Self { location_names: vec![], test_names: vec![] }
    }

    pub fn new(
        location_names: Vec<RoswaalLocationName>,
        test_names: Vec<String>
    ) -> Self {
        Self { location_names, test_names }
    }
}

/// A trait for self-initializing by compiling roswaal test syntax.
pub trait RoswaalCompile: Sized {
    fn compile_syntax(
        syntax: RoswaalTestSyntax,
        ctx: RoswaalCompileContext
    ) -> Result<Self, Vec<RoswaalCompilationError>>;

    fn compile(
        source_code: &str,
        ctx: RoswaalCompileContext
    ) -> Result<Self, Vec<RoswaalCompilationError>> {
        Self::compile_syntax(RoswaalTestSyntax::from(source_code), ctx)
    }
}

impl RoswaalCompile for RoswaalTest {
    fn compile_syntax(
        syntax: RoswaalTestSyntax,
        ctx: RoswaalCompileContext
    ) -> Result<Self, Vec<RoswaalCompilationError>> {
        let mut errors: Vec<RoswaalCompilationError> = Vec::new();
        let mut has_test_line = false;
        for line in syntax.token_lines() {
            let line_number = line.line_number();
            match line.token() {
                RoswaalTestSyntaxToken::NewTest { name } => {
                    has_test_line = true;
                    if ctx.test_names.contains(&name.to_string()) {
                        let error = RoswaalCompilationError {
                            line_number,
                            code: RoswaalCompilationErrorCode::DuplicateTestName(name.to_string())
                        };
                        errors.push(error);
                    }
                    if has_test_line {
                        let error = RoswaalCompilationError {
                            line_number,
                            code: RoswaalCompilationErrorCode::TestNameAlreadyDeclared
                        };
                        errors.push(error);
                    }
                },
                RoswaalTestSyntaxToken::Step { name, description: _ } => {
                    let error = RoswaalCompilationError {
                        line_number,
                        code: RoswaalCompilationErrorCode::NoCommandDescription {
                            command_name: name.to_string()
                        }
                    };
                    errors.push(error);
                },
                RoswaalTestSyntaxToken::SetLocation { parse_result } => {
                    let err = match parse_result {
                        Ok(name) => RoswaalCompilationError {
                            line_number,
                            code: RoswaalCompilationErrorCode::InvalidLocationName(
                                name.name().to_string()
                            )
                        },
                        Err(_) => RoswaalCompilationError {
                            line_number,
                            code: RoswaalCompilationErrorCode::NoCommandDescription {
                                command_name: "Set Location".to_string()
                            }
                        }
                    };
                    errors.push(err);
                },
                RoswaalTestSyntaxToken::UnknownCommand { name, description: _ } => {
                    let error = RoswaalCompilationError {
                        line_number,
                        code: RoswaalCompilationErrorCode::InvalidCommandName(
                            name.to_string()
                        )
                    };
                    errors.push(error)
                },
                RoswaalTestSyntaxToken::Unknown { source } => {
                    let error = RoswaalCompilationError {
                        line_number,
                        code: RoswaalCompilationErrorCode::InvalidCommandName(
                            source.to_string()
                        )
                    };
                    errors.push(error)
                },
                _ => {}
            }
        }
        if !has_test_line {
            let error = RoswaalCompilationError {
                line_number: syntax.last_line_number(),
                code: RoswaalCompilationErrorCode::NoTestName
            };
            errors.push(error)
        } else {
            let error = RoswaalCompilationError {
                line_number: syntax.last_line_number(),
                code: RoswaalCompilationErrorCode::NoTestSteps
            };
            errors.push(error);
        }
        Err(errors)
    }
}

#[cfg(test)]
mod compiler_tests {
    use std::str::FromStr;

    use crate::language::location::RoswaalLocationName;

    use super::*;

    #[test]
    fn test_parse_returns_no_name_for_empty_string() {
        let result = RoswaalTest::compile("", RoswaalCompileContext::empty());
        let error = RoswaalCompilationError {
            line_number: 1,
            code: RoswaalCompilationErrorCode::NoTestName
        };
        assert_contains_compile_error(&result, &error);
    }

    #[test]
    fn test_parse_returns_no_name_for_random_multiline_string() {
        let test = "\n\n\n\n";
        let result = RoswaalTest::compile(test, RoswaalCompileContext::empty());
        let error = RoswaalCompilationError {
            line_number: 4,
            code: RoswaalCompilationErrorCode::NoTestName
        };
        assert_contains_compile_error(&result, &error);
    }

    #[test]
    fn test_parse_returns_unknown_command_and_no_test_name_for_random_string() {
        let code = "jkashdkjashdkjahsd ehiuh3ui2geuyg23urg";
        let result = RoswaalTest::compile(code, RoswaalCompileContext::empty());
        let errors = vec!(
            RoswaalCompilationError {
                line_number: 1,
                code: RoswaalCompilationErrorCode::InvalidCommandName(
                    code.to_string()
                )
            },
            RoswaalCompilationError {
                line_number: 1,
                code: RoswaalCompilationErrorCode::NoTestName
            }
        );
        assert_contains_compile_error(&result, &errors[0]);
        assert_contains_compile_error(&result, &errors[1])
    }

    #[test]
    fn test_compile_does_not_return_a_no_test_steps_error_when_no_test_name() {
        let result = RoswaalTest::compile("", RoswaalCompileContext::empty());
        let error = RoswaalCompilationError {
            line_number: 1,
            code: RoswaalCompilationErrorCode::NoTestSteps
        };
        assert_not_contains_compile_error(&result, &error)
    }

    #[test]
    fn test_parse_returns_no_steps_when_name_formatted_correctly_uppercase() {
        let result = RoswaalTest::compile("New Test: Hello world", RoswaalCompileContext::empty());
        let error = RoswaalCompilationError {
            line_number: 1,
            code: RoswaalCompilationErrorCode::NoTestSteps
        };
        assert_contains_compile_error(&result, &error);
    }

    #[test]
    fn test_parse_returns_no_steps_when_name_formatted_correctly_lowercase() {
        let result = RoswaalTest::compile("new test: Hello world", RoswaalCompileContext::empty());
        let error = RoswaalCompilationError {
            line_number: 1,
            code: RoswaalCompilationErrorCode::NoTestSteps
        };
        assert_contains_compile_error(&result, &error);
    }

    #[test]
    fn test_parse_returns_invalid_command_name_and_no_steps_when_random_string_after_new_test() {
        let test = "\
new test: Hello world
lsjkhadjkhasdfjkhasdjkfhkjsd
";
        let errors = vec!(
            RoswaalCompilationError {
                line_number: 2,
                code: RoswaalCompilationErrorCode::InvalidCommandName(
                    "lsjkhadjkhasdfjkhasdjkfhkjsd".to_string()
                )
            },
            RoswaalCompilationError {
                line_number: 2,
                code: RoswaalCompilationErrorCode::NoTestSteps
            }
        );
        let result = RoswaalTest::compile(test, RoswaalCompileContext::empty());
        assert_contains_compile_error(&result, &errors[0]);
        assert_contains_compile_error(&result, &errors[1])
    }

    #[test]
    fn test_parse_returns_no_steps_when_mutliple_lines_before_new_test() {
        let test = "\n\n\n\nnew test: Hello world";
        let error = RoswaalCompilationError {
            line_number: 5,
            code: RoswaalCompilationErrorCode::NoTestSteps
        };
        let result = RoswaalTest::compile(test, RoswaalCompileContext::empty());
        assert_contains_compile_error(&result, &error)
    }

    #[test]
    fn test_parse_returns_duplicate_test_name_when_new_test_matches_existing_test_name() {
        let test = "new test: Test 1";
        let test_name = "Test 1";
        let result = RoswaalTest::compile(
            test,
            RoswaalCompileContext::new(vec![], vec![test_name.to_string()])
        );
        let error = RoswaalCompilationError {
            line_number: 1,
            code: RoswaalCompilationErrorCode::DuplicateTestName(test_name.to_string())
        };
        assert_contains_compile_error(&result, &error)
    }

    #[test]
    fn test_parse_returns_test_name_declared_when_2_new_test_commands() {
        let test = "\
New test: Test 1
New Test: Test 2
";
        let result = RoswaalTest::compile(test, RoswaalCompileContext::empty());
        let error = RoswaalCompilationError {
            line_number: 2,
            code: RoswaalCompilationErrorCode::TestNameAlreadyDeclared
        };
        assert_contains_compile_error(&result, &error)
    }

    #[test]
    fn test_parse_returns_invalid_command_name_when_command_is_not_a_step() {
        let test = "\
new test: Hello world
passo 1: mamma mia
";
        let result = RoswaalTest::compile(test, RoswaalCompileContext::empty());
        let step_name = "passo 1".to_string();
        let error = RoswaalCompilationError {
            line_number: 2,
            code: RoswaalCompilationErrorCode::InvalidCommandName(step_name)
        };
        assert_contains_compile_error(&result, &error);
    }

    #[test]
    fn test_parse_returns_no_step_description_when_step_lacks_description() {
        let test = "\
New Test: Hello wordl
Step 1:
";
        let result = RoswaalTest::compile(test, RoswaalCompileContext::empty());
        let error = RoswaalCompilationError {
            line_number: 2,
            code: RoswaalCompilationErrorCode::NoCommandDescription {
                command_name: "Step 1".to_string()
            }
        };
        assert_contains_compile_error(&result, &error);
    }

    #[test]
    fn test_parse_returns_no_location_specified_when_location_step_is_empty() {
        let test = "\
New test: This is an acceptance test
Set Location:
";
        let result = RoswaalTest::compile(test, RoswaalCompileContext::empty());
        let error = RoswaalCompilationError {
            line_number: 2,
            code: RoswaalCompilationErrorCode::NoCommandDescription {
                command_name: "Set Location".to_string()
            }
        };
        assert_contains_compile_error(&result, &error);
    }

    #[test]
    fn test_parse_returns_invalid_location_when_location_name_is_not_in_locations_list() {
        let location_names = vec!(RoswaalLocationName::from_str("Hello").unwrap());
        let test = "\
New test: This is an acceptance test
Set Location: world
";
        let result = RoswaalTest::compile(
            test,
            RoswaalCompileContext { location_names, test_names: vec![] }
        );
        let error = RoswaalCompilationError {
            line_number: 2,
            code: RoswaalCompilationErrorCode::InvalidLocationName("world".to_string())
        };
        assert_contains_compile_error(&result, &error);
    }

    fn assert_contains_compile_error(
        result: &Result<RoswaalTest, Vec<RoswaalCompilationError>>,
        error: &RoswaalCompilationError
    ) {
        assert!(result.as_ref().err().unwrap().contains(error))
    }

    fn assert_not_contains_compile_error(
        result: &Result<RoswaalTest, Vec<RoswaalCompilationError>>,
        error: &RoswaalCompilationError
    ) {
        assert!(!result.as_ref().err().unwrap().contains(error))
    }
}
