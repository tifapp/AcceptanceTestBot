use super::{ast::{RoswaalTestSyntax, RoswaalTestSyntaxCommand, RoswaalTestSyntaxLineContent}, location::RoswaalLocationName, test::{RoswaalTest, RoswaalTestCommand}};

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
    NoStepRequirement { step_name: String, step_description: String },
    NoRequirementStep { requirement_name: String, requirement_description: String },
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

struct CommandInfo {
    line_number: u32,
    name: String,
    description: String
}

struct CompileContext {
    location_names: Vec<RoswaalLocationName>,
    test_names: Vec<String>,
    errors: Vec<RoswaalCompilationError>,
    test_name: Option<String>,
    step_name: Vec<CommandInfo>,
    requirement_name: Vec<CommandInfo>,
    commnands: Vec<RoswaalTestCommand>
}

impl CompileContext {
    fn new(ctx: RoswaalCompileContext) -> Self {
        CompileContext {
            location_names: ctx.location_names,
            test_names: ctx.test_names,
            errors: vec![],
            test_name: None,
            step_name: vec![],
            requirement_name: vec![],
            commnands: vec![]
        }
    }
}

impl CompileContext {
    fn append_error(&mut self, line_number: u32, code: RoswaalCompilationErrorCode) {
        self.errors.push(RoswaalCompilationError { line_number, code })
    }

    fn try_set_test_name(&mut self, line_number: u32, name: &str) {
        let name = name.to_string();
        if self.test_names.contains(&name) {
            self.append_error(line_number, RoswaalCompilationErrorCode::DuplicateTestName(name));
        } else if self.test_name.is_some() {
            self.append_error(line_number, RoswaalCompilationErrorCode::TestNameAlreadyDeclared);
        } else {
            self.test_name = Some(name);
        }
    }

    fn append_step(&mut self, line_number: u32, name: &str, description: &str) {
        if let Some(requirement_name) = self.requirement_name.pop() {
            let command = RoswaalTestCommand::Step {
                name: description.to_string(),
                requirement: requirement_name.description
            };
            self.commnands.push(command)
        } else {
            let info = CommandInfo {
                line_number,
                name: name.to_string(),
                description: description.to_string()
            };
            self.step_name.push(info);
        }
    }

    fn append_requirment(&mut self, line_number: u32, name: &str, description: &str) {
        if let Some(step_name) = self.step_name.pop() {
            let command = RoswaalTestCommand::Step {
                name: step_name.description,
                requirement: description.to_string()
            };
            self.commnands.push(command)
        } else {
            let info = CommandInfo {
                line_number,
                name: name.to_string(),
                description: description.to_string()
            };
            self.requirement_name.push(info);
        }
    }

    fn test(self) -> Result<RoswaalTest, Vec<RoswaalCompilationError>> {
        let test_name = match self.test_name {
            Some(name) => name,
            _ => return Err(self.errors)
        };
        if !self.errors.is_empty() {
            return Err(self.errors)
        }
        return Ok(RoswaalTest::new(test_name, self.commnands));
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
        let mut ctx = CompileContext::new(ctx);
        for line in syntax.lines() {
            let line_number = line.line_number();
            match line.content() {
                RoswaalTestSyntaxLineContent::Command { name, description, command } => {
                    if description.is_empty() {
                        let code = RoswaalCompilationErrorCode::NoCommandDescription {
                            command_name: name.to_string()
                        };
                        ctx.append_error(line_number, code);
                        continue
                    }
                    match command {
                        RoswaalTestSyntaxCommand::NewTest => {
                            ctx.try_set_test_name(line_number, description);
                        },
                        RoswaalTestSyntaxCommand::SetLocation { parse_result } => {
                            if let Some(location_name) = parse_result.as_ref().ok() {
                                let code =  RoswaalCompilationErrorCode::InvalidLocationName(
                                    location_name.name().to_string()
                                );
                                ctx.append_error(line_number, code);
                            }
                        },
                        RoswaalTestSyntaxCommand::UnknownCommand => {
                            let code = RoswaalCompilationErrorCode::InvalidCommandName(
                                name.to_string()
                            );
                            ctx.append_error(line_number, code);
                        },
                        RoswaalTestSyntaxCommand::Step => {
                            ctx.append_step(line_number, name, description);
                        },
                        RoswaalTestSyntaxCommand::Requirement => {
                            ctx.append_requirment(line_number, name, description);
                        },
                        _ => {}
                    }
                }
                RoswaalTestSyntaxLineContent::Unknown(content) => {
                    let code = RoswaalCompilationErrorCode::InvalidCommandName(
                        content.to_string()
                    );
                    ctx.append_error(line_number, code);
                }
            }
        }
        if ctx.test_name.is_none() {
            ctx.append_error(syntax.last_line_number(), RoswaalCompilationErrorCode::NoTestName);
        } else if ctx.commnands.is_empty() {
            ctx.append_error(syntax.last_line_number(), RoswaalCompilationErrorCode::NoTestSteps);
        }
        if let Some(step_name) = ctx.step_name.first() {
            let code = RoswaalCompilationErrorCode::NoStepRequirement {
                step_name: step_name.name.clone(),
                step_description: step_name.description.clone()
            };
            ctx.append_error(step_name.line_number, code);
        }
        if let Some(requirment_name) = ctx.requirement_name.first() {
            let code = RoswaalCompilationErrorCode::NoRequirementStep {
                requirement_name: requirment_name.name.clone(),
                requirement_description: requirment_name.description.clone()
            };
            ctx.append_error(requirment_name.line_number, code);
        }
        ctx.test()
    }
}

#[cfg(test)]
mod compiler_tests {
    use std::str::FromStr;

    use crate::language::{location::RoswaalLocationName, test::RoswaalTestCommand};

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
        let result = RoswaalTest::compile(
            "New Test: Hello world",
            RoswaalCompileContext::empty()
        );
        let error = RoswaalCompilationError {
            line_number: 1,
            code: RoswaalCompilationErrorCode::NoTestSteps
        };
        assert_contains_compile_error(&result, &error);
    }

    #[test]
    fn test_parse_returns_no_steps_when_name_formatted_correctly_lowercase() {
        let result = RoswaalTest::compile(
            "new test: Hello world",
            RoswaalCompileContext::empty()
        );
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
            code: RoswaalCompilationErrorCode::DuplicateTestName(
                test_name.to_string()
            )
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
            RoswaalCompileContext::new(location_names, vec![])
        );
        let error = RoswaalCompilationError {
            line_number: 2,
            code: RoswaalCompilationErrorCode::InvalidLocationName("world".to_string())
        };
        assert_contains_compile_error(&result, &error);
    }

    #[test]
    fn test_parse_returns_no_step_requirement_when_step_does_not_have_requirement() {
        let test = "\
New Test: I am a test
Step 1: Jump in the air
";
        let result = RoswaalTest::compile(test, RoswaalCompileContext::empty());
        let error = RoswaalCompilationError {
            line_number: 2,
            code: RoswaalCompilationErrorCode::NoStepRequirement {
                step_name: "Step 1".to_string(),
                step_description: "Jump in the air".to_string()
            }
        };
        assert_contains_compile_error(&result, &error);
    }

    #[test]
    fn test_parse_returns_no_requirement_step_when_requirement_does_not_have_step() {
        let test = "\
New Test: I am a test
Requirement 1: Jump in the air
";
        let result = RoswaalTest::compile(test, RoswaalCompileContext::empty());
        let error = RoswaalCompilationError {
            line_number: 2,
            code: RoswaalCompilationErrorCode::NoRequirementStep {
                requirement_name: "Requirement 1".to_string(),
                requirement_description: "Jump in the air".to_string()
            }
        };
        assert_contains_compile_error(&result, &error);
    }

    #[test]
    fn test_parse_returns_test_with_single_step() {
        let test = "\
New Test: Piccolo fights cyborgs
Step 1: Piccolo can use special-beam-cannon
Requirement 1: Have Piccolo charge his special-beam-cannon
";
        let result = RoswaalTest::compile(test, RoswaalCompileContext::empty()).unwrap();
        let expected_test = RoswaalTest::new(
            "Piccolo fights cyborgs".to_string(),
            vec![
                RoswaalTestCommand::Step {
                    name: "Piccolo can use special-beam-cannon".to_string(),
                    requirement: "Have Piccolo charge his special-beam-cannon".to_string()
                }
            ]
        );
        assert_eq!(result, expected_test)
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
