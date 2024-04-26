use std::{iter::Enumerate, str::{FromStr, Lines}};

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
    Step { description: &'a str },
    /// A line denoting the "Abstract" command.
    Abstract { description: &'a str },
    /// A line denoting the "New Test" command.
    NewTest { name: &'a str },
    /// A line denoting the "SetLocation" command.
    SetLocation { parse_result: RoswaalLocationParsingResult },
    /// A line denoting the "Requirement" command that is to be paired with a
    /// respective step command.
    Requirement { description: &'a str },
    /// A line which has proper command syntax, but the command is not known.
    UnknownCommand { name: &'a str, description: &'a str },
    /// A line which does not follow traditional command syntax.
    Unknown { source: &'a str },
    /// An empty line.
    EmptyLine
}

impl <'a> From<&'a str> for RoswaalTestSyntaxToken<'a> {
    fn from(line: &'a str) -> Self {
        let (command, description) = match line.split_once(":") {
            Some(split) => split,
            None => {
                return if line.is_empty() {
                    Self::EmptyLine
                } else {
                    Self::Unknown { source: line }
                }
            }
        };
        let normalized_command = command.roswaal_normalize();
        let description = description.trim();
        if normalized_command.starts_with("step") {
            return Self::Step { description }
        } else if normalized_command.starts_with("setlocation") {
            return Self::SetLocation {
                parse_result: RoswaalLocationName::from_str(&description)
            }
        } else if normalized_command.starts_with("newtest") {
            return Self::NewTest { name: description }
        } else if normalized_command.starts_with("requirement") {
            return Self::Requirement { description }
        } else if normalized_command.starts_with("abstract") {
            return Self::Abstract { description }
        } else {
            return Self::UnknownCommand {
                name: command,
                description
            }
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
    /// Returns the original source code of represented by the test syntax.
    pub fn source_code(&self) -> &str {
        &self.source_code
    }

    /// Returns an iterator of syntax tokens for each line in the source code.
    pub fn token_lines(&self) -> RoswaalTestTokenLines {
        RoswaalTestTokenLines { lines: self.source_code().lines().enumerate() }
    }
}

impl From<&str> for RoswaalTestSyntax {
    fn from(source_code: &str) -> Self {
        Self { source_code: source_code.to_string() }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct RoswaalTestSyntaxLine<'a> {
    pub line_number: u32,
    pub token: RoswaalTestSyntaxToken<'a>
}

/// An iterator of syntax tokens for each line in the source code.
pub struct RoswaalTestTokenLines<'a> {
    lines: Enumerate<Lines<'a>>
}

impl <'a> Iterator for RoswaalTestTokenLines<'a> {
    type Item = RoswaalTestSyntaxLine<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.lines.next().map(|(i, line)| {
            RoswaalTestSyntaxLine {
                line_number: (i + 1) as u32,
                token: RoswaalTestSyntaxToken::from(line)
            }
        })
    }
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
            let token = RoswaalTestSyntaxToken::from("");
            assert_eq!(token, RoswaalTestSyntaxToken::EmptyLine)
        }

        #[test]
        fn test_from_string_returns_unknown_for_jibberish() {
            let source = "Sorry King Kai! I didn't know where else to bring him!";
            let token = RoswaalTestSyntaxToken::from(source);
            assert_eq!(token, RoswaalTestSyntaxToken::Unknown { source })
        }

        #[test]
        fn test_from_string_returns_step_for_simple_step_commands() {
            fn assert_step_description(line: &str, description: &str) {
                let token = RoswaalTestSyntaxToken::from(line);
                let expected_token = RoswaalTestSyntaxToken::Step {
                    description
                };
                assert_eq!(token, expected_token)
            }
            assert_step_description(
                "Step 1: Hello world this is a test",
                "Hello world this is a test"
            );
            assert_step_description(
                "Step: i am batman",
                "i am batman"
            );
            assert_step_description(
                "step 1: Hello",
                "Hello"
            );
            assert_step_description(
                "   step    1   : Hello",
                "Hello"
            );
            assert_step_description(
                "step one: Hello world this is a test",
                "Hello world this is a test"
            );
            assert_step_description(
                "step:",
                ""
            )
        }

        #[test]
        fn test_from_string_returns_set_location_for_set_location_commands() {
            fn assert_set_location_name(line: &str, name: &str) {
                let token = RoswaalTestSyntaxToken::from(line);
                let parse_result = RoswaalLocationName::from_str(name);
                let expected_token = RoswaalTestSyntaxToken::SetLocation {
                    parse_result: RoswaalLocationName::from_str(name)
                };
                assert_eq!(token, expected_token);
                assert_eq!(name, parse_result.unwrap().name())
            }

            fn assert_no_location_name(
                line: &str,
                error: RoswaalLocationNameParsingError
            ) {
                let token = RoswaalTestSyntaxToken::from(line);
                let expected_token = RoswaalTestSyntaxToken::SetLocation {
                    parse_result: Err(error)
                };
                assert_eq!(token, expected_token)
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
                let token = RoswaalTestSyntaxToken::from(line);
                let expected_token = RoswaalTestSyntaxToken::UnknownCommand {
                    name,
                    description
                };
                assert_eq!(token, expected_token)
            }

            assert_unknown_command("dkjhkjdh: hello", "dkjhkjdh", "hello");
            assert_unknown_command("dkjh kjdh: hello ", "dkjh kjdh", "hello")
        }

        #[test]
        fn test_from_string_returns_new_test_for_new_test_command() {
            fn assert_new_test(line: &str, name: &str) {
                let token = RoswaalTestSyntaxToken::from(line);
                let expected_token = RoswaalTestSyntaxToken::NewTest {
                    name
                };
                assert_eq!(token, expected_token)
            }

            assert_new_test("New Test: Hello world", "Hello world");
            assert_new_test("new test: test", "test");
            assert_new_test(" new    tESt    : weird  ", "weird")
        }

        #[test]
        fn test_from_string_returns_requirement_for_requirement_command() {
            fn assert_requirement(line: &str, description: &str) {
                let token = RoswaalTestSyntaxToken::from(line);
                let expected_token = RoswaalTestSyntaxToken::Requirement {
                    description
                };
                assert_eq!(token, expected_token)
            }

            assert_requirement("Requirement 1: Hello world", "Hello world");
            assert_requirement("requirement: test", "test");
            assert_requirement(" requirement   4: weird  ", "weird")
        }

        #[test]
        fn test_from_string_returns_abstract_for_abstract_command() {
            fn assert_abstract(line: &str, description: &str) {
                let token = RoswaalTestSyntaxToken::from(line);
                let expected_token = RoswaalTestSyntaxToken::Abstract {
                    description
                };
                assert_eq!(token, expected_token)
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
                            description: "Write a step"
                        }
                    },
                    RoswaalTestSyntaxLine {
                        line_number: 3,
                        token: RoswaalTestSyntaxToken::Step {
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
                        line_number: 6,
                        token: RoswaalTestSyntaxToken::EmptyLine
                    },
                    RoswaalTestSyntaxLine {
                        line_number: 7,
                        token: RoswaalTestSyntaxToken::Requirement {
                            description: "Do the thing"
                        }
                    },
                    RoswaalTestSyntaxLine {
                        line_number: 8,
                        token: RoswaalTestSyntaxToken::Requirement {
                            description: "Do the other thing"
                        }
                    }
                )
            )
        }
    }
}
