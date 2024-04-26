use std::str::FromStr;

use super::{
    location::{RoswaalLocationName, RoswaalLocationParsingResult},
    normalize::RoswaalNormalize
};

/// A token of roswaal test syntax.
///
/// Each token represents a line of source code. See `RoswaalTestSyntax`.
#[derive(Debug, PartialEq, Eq)]
pub enum RoswaalTestSyntaxToken<'a> {
    /// A line denoting a "Step" command without its matching "Requirement"
    /// command.
    Step { name: &'a str, description: &'a str },
    /// A line denoting the "Abstract" command.
    Abstract { description: &'a str },
    /// A line denoting the "New Test" command.
    NewTest { name: &'a str },
    /// A line denoting the "SetLocation" command.
    SetLocation { parse_result: RoswaalLocationParsingResult },
    /// A line denoting the "Requirement" command that is to be paired with a
    /// respective step command.
    Requirement { name: &'a str, description: &'a str },
    /// A line which has proper command syntax, but the command is not known.
    UnknownCommand { name: &'a str, description: &'a str },
    /// A line which does not follow traditional command syntax.
    Unknown { source: &'a str }
}

impl <'a> TryFrom<&'a str> for RoswaalTestSyntaxToken<'a> {
    type Error = ();

    fn try_from(line: &'a str) -> Result<Self, ()> {
        let (command, description) = match line.split_once(":") {
            Some(split) => split,
            None => {
                return if line.is_empty() {
                    Err(())
                } else {
                    Ok(Self::Unknown { source: line })
                }
            }
        };
        let normalized_command = command.roswaal_normalize();
        let description = description.trim();
        if normalized_command.starts_with("step") {
            return Ok(Self::Step { name: command, description })
        } else if normalized_command.starts_with("setlocation") {
            return Ok(
                Self::SetLocation {
                    parse_result: RoswaalLocationName::from_str(&description)
                }
            )
        } else if normalized_command.starts_with("newtest") {
            return Ok(Self::NewTest { name: description })
        } else if normalized_command.starts_with("requirement") {
            return Ok(Self::Requirement { name: command, description })
        } else if normalized_command.starts_with("abstract") {
            return Ok(Self::Abstract { description })
        } else {
            return Ok(Self::UnknownCommand { name: command, description })
        }
    }
}

/// An opaque data structure representing parsed roswaal test syntax.
///
/// The syntax of a test is linear with each token representing a single line,
/// and contains no nested structures. Each token is split into a command and
/// description by a ":". Command names are case and whitespace insensitive as
/// to make writing test specifications as natural as possible. For example,
/// "step 1" and "Step" will both be parsed as a step command.
///
/// The primary token is a "step" which describes what the test should do from
/// the perspective of an end-user. Each step is paired with a matching
/// requirement token, which serves as a techincal explanation on what to do to
/// implement the step. Once compiled to typescript, the requirement token
/// description represents a function name in the resulting generated code with
/// the step description as a documentation comment.
///
/// Other semantic tokens exist that will generate common code used in tests
/// like "Set Location" which sets the device's location to the area specified
/// by the token.
///
/// Example Syntax (creating a test specification):
/// ```
/// New Test: My cool test
/// Step 1: I am a step
/// Step 2: This is another step
/// Set Location: Antarctica
/// Requirement 1: I am a requirement that matches step 1.
/// Requirement 2: I am a requirement that is paired with step 2.
/// ```
pub struct RoswaalTestSyntax {
    source_code: String
}

impl RoswaalTestSyntax {
    /// Returns an iterator of syntax tokens for each line in the source code.
    pub fn token_lines(&self) -> impl Iterator<Item = RoswaalTestSyntaxLine> {
        self.source_code.lines()
            .enumerate()
            .filter_map(|(i, line)| {
                RoswaalTestSyntaxToken::try_from(line)
                    .ok()
                    .map(|token| {
                        RoswaalTestSyntaxLine {
                            line_number: (i + 1) as u32,
                            token
                        }
                    })
            })
    }

    /// Returns the last line number of this syntax.
    ///
    /// If the syntax is empty, line 1 is returned.
    pub fn last_line_number(&self) -> u32 {
        let line_count = self.source_code.lines().count();
        return if line_count == 0 {
            1
        } else {
            line_count as u32
        }
    }
}

impl From<&str> for RoswaalTestSyntax {
    fn from(source_code: &str) -> Self {
        Self { source_code: source_code.to_string() }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct RoswaalTestSyntaxLine<'a> {
    line_number: u32,
    token: RoswaalTestSyntaxToken<'a>
}

impl <'a> RoswaalTestSyntaxLine<'a> {
    pub fn line_number(&self) -> u32 { self.line_number }
    pub fn token(&self) -> &RoswaalTestSyntaxToken<'a> { &self.token }
}

#[cfg(test)]
mod ast_tests {
    use super::*;

    #[cfg(test)]
    mod token_tests {
        use std::str::FromStr;

        use crate::language::location::RoswaalLocationNameParsingError;

        use super::*;

        #[test]
        fn test_from_string_returns_empty_line_for_empty_string() {
            assert_eq!(RoswaalTestSyntaxToken::try_from(""), Err(()))
        }

        #[test]
        fn test_from_string_returns_unknown_for_jibberish() {
            let source = "Sorry King Kai! I didn't know where else to bring him!";
            let token = RoswaalTestSyntaxToken::try_from(source);
            assert_eq!(token, Ok(RoswaalTestSyntaxToken::Unknown { source }))
        }

        #[test]
        fn test_from_string_returns_step_for_simple_step_commands() {
            fn assert_step_description(line: &str, name: &str, description: &str) {
                let expected_token = RoswaalTestSyntaxToken::Step {
                    name,
                    description
                };
                assert_eq!(RoswaalTestSyntaxToken::try_from(line), Ok(expected_token))
            }
            assert_step_description(
                "Step 1: Hello world this is a test",
                "Step 1",
                "Hello world this is a test"
            );
            assert_step_description(
                "Step: i am batman",
                "Step",
                "i am batman"
            );
            assert_step_description(
                "step 1: Hello",
                "step 1",
                "Hello"
            );
            assert_step_description(
                "   step    1   : Hello",
                "   step    1   ",
                "Hello"
            );
            assert_step_description(
                "step one: Hello world this is a test",
                "step one",
                "Hello world this is a test"
            );
            assert_step_description(
                "step:",
                "step",
                ""
            )
        }

        #[test]
        fn test_from_string_returns_set_location_for_set_location_commands() {
            fn assert_set_location_name(line: &str, name: &str) {
                let parse_result = RoswaalLocationName::from_str(name);
                let expected_token = RoswaalTestSyntaxToken::SetLocation {
                    parse_result: RoswaalLocationName::from_str(name)
                };
                assert_eq!(RoswaalTestSyntaxToken::try_from(line), Ok(expected_token));
                assert_eq!(name, parse_result.unwrap().name())
            }

            fn assert_no_location_name(
                line: &str,
                error: RoswaalLocationNameParsingError
            ) {
                let expected_token = RoswaalTestSyntaxToken::SetLocation {
                    parse_result: Err(error)
                };
                assert_eq!(RoswaalTestSyntaxToken::try_from(line), Ok(expected_token))
            }

            assert_set_location_name("set location: Apple", "Apple");
            assert_set_location_name("Set Location: Houston", "Houston");
            assert_set_location_name("Set      location: A", "A");
            assert_set_location_name("Set      location      : A", "A");
            assert_set_location_name("   Set      location      : A", "A");
            assert_no_location_name(
                "set location:",
                RoswaalLocationNameParsingError::Empty
            )
        }

        #[test]
        fn test_from_string_returns_unknown_command_for_random_commands() {
            fn assert_unknown_command(line: &str, name: &str, description: &str) {
                let expected_token = RoswaalTestSyntaxToken::UnknownCommand {
                    name,
                    description
                };
                assert_eq!(RoswaalTestSyntaxToken::try_from(line), Ok(expected_token))
            }

            assert_unknown_command("dkjhkjdh: hello", "dkjhkjdh", "hello");
            assert_unknown_command("dkjh kjdh: hello ", "dkjh kjdh", "hello")
        }

        #[test]
        fn test_from_string_returns_new_test_for_new_test_command() {
            fn assert_new_test(line: &str, name: &str) {
                let expected_token = RoswaalTestSyntaxToken::NewTest {
                    name
                };
                assert_eq!(RoswaalTestSyntaxToken::try_from(line), Ok(expected_token))
            }

            assert_new_test("New Test: Hello world", "Hello world");
            assert_new_test("new test: test", "test");
            assert_new_test(" new    tESt    : weird  ", "weird")
        }

        #[test]
        fn test_from_string_returns_requirement_for_requirement_command() {
            fn assert_requirement(line: &str, name: &str, description: &str) {
                let expected_token = RoswaalTestSyntaxToken::Requirement {
                    name,
                    description
                };
                assert_eq!(RoswaalTestSyntaxToken::try_from(line), Ok(expected_token))
            }

            assert_requirement(
                "Requirement 1: Hello world",
                "Requirement 1",
                "Hello world"
            );
            assert_requirement(
                "requirement: test",
                "requirement",
                "test"
            );
            assert_requirement(
                " requirement   4: weird  ",
                " requirement   4",
                "weird"
            )
        }

        #[test]
        fn test_from_string_returns_abstract_for_abstract_command() {
            fn assert_abstract(line: &str, description: &str) {
                let expected_token = RoswaalTestSyntaxToken::Abstract {
                    description
                };
                assert_eq!(RoswaalTestSyntaxToken::try_from(line), Ok(expected_token))
            }

            assert_abstract("Abstract 1: Hello world", "Hello world");
            assert_abstract("abstract: test", "test");
            assert_abstract(" abstract   4: weird  ", "weird")
        }
    }

    #[cfg(test)]
    mod syntax_tests {
        use std::str::FromStr;

        use crate::language::ast::{ast_tests::RoswaalLocationName, RoswaalTestSyntaxLine};

        use super::{RoswaalTestSyntax, RoswaalTestSyntaxToken};

        #[test]
        fn test_last_line_number_returns_1_when_empty() {
            let syntax = RoswaalTestSyntax::from("");
            assert_eq!(syntax.last_line_number(), 1)
        }

        #[test]
        fn test_last_line_number_returns_1_when_single_line() {
            let syntax = RoswaalTestSyntax::from("hello");
            assert_eq!(syntax.last_line_number(), 1)
        }

        #[test]
        fn test_last_line_number_returns_1_number_of_lines_in_code() {
            let test = "\
New Test: I am a test
Step 1: Piccolo was the first to try
Requirement 1: Have piccolo fight Android 17 and 18 all at once
Step 2: And the first to die
Requirement 2: Make sure that \"even Krillin\" can't be stopped by the dreadful duo
";
            let syntax = RoswaalTestSyntax::from(test);
            assert_eq!(syntax.last_line_number(), 5)
        }

        #[test]
        fn test_token_lines_iterator() {
            let test = "\
New Test: Something cool
Step 1: Write a step
Step 2: Another step
Set Location: Europe
Big: chungus

Requirement 1: Do the thing
Requirement 2: Do the other thing
";
            let syntax = RoswaalTestSyntax::from(test);
            let tokens = syntax
                .token_lines()
                .collect::<Vec<RoswaalTestSyntaxLine>>();
            assert_eq!(
                tokens,
                vec!(
                    RoswaalTestSyntaxLine {
                        line_number: 1,
                        token: RoswaalTestSyntaxToken::NewTest {
                            name: "Something cool"
                        }
                    },
                    RoswaalTestSyntaxLine {
                        line_number: 2,
                        token: RoswaalTestSyntaxToken::Step {
                            name: "Step 1",
                            description: "Write a step"
                        }
                    },
                    RoswaalTestSyntaxLine {
                        line_number: 3,
                        token: RoswaalTestSyntaxToken::Step {
                            name: "Step 2",
                            description: "Another step"
                        }
                    },
                    RoswaalTestSyntaxLine {
                        line_number: 4,
                        token: RoswaalTestSyntaxToken::SetLocation {
                            parse_result: RoswaalLocationName::from_str("Europe")
                        }
                    },
                    RoswaalTestSyntaxLine {
                        line_number: 5,
                        token: RoswaalTestSyntaxToken::UnknownCommand {
                            name: "Big",
                            description: "chungus"
                        }
                    },
                    RoswaalTestSyntaxLine {
                        line_number: 7,
                        token: RoswaalTestSyntaxToken::Requirement {
                            name: "Requirement 1",
                            description: "Do the thing"
                        }
                    },
                    RoswaalTestSyntaxLine {
                        line_number: 8,
                        token: RoswaalTestSyntaxToken::Requirement {
                            name: "Requirement 2",
                            description: "Do the other thing"
                        }
                    }
                )
            )
        }
    }
}
