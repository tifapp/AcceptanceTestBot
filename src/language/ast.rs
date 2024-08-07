use std::str::FromStr;

use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};

use crate::location::name::{RoswaalLocationName, RoswaalLocationParsingResult};

/// A token of roswaal test syntax.
///
/// Each token represents a line of source code. See `RoswaalTestSyntax`.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RoswaalTestSyntaxCommand<'a> {
    /// A line denoting a "Step" command without its matching "Requirement"
    /// command.
    Step { label: &'a str },
    /// A line denoting the "Abstract" command.
    Abstract,
    /// A line denoting the "New Test" command.
    NewTest,
    /// A line denoting the "Set Location" command.
    SetLocation {
        parse_result: RoswaalLocationParsingResult,
    },
    /// A line denoting the "Requirement" command that is to be paired with a
    /// respective step command.
    Requirement { label: &'a str },
    /// A line which has proper command syntax, but the command is not known.
    UnknownCommand,
}

static KNOWN_COMMANDS_REGEX: Lazy<Regex> = Lazy::new(|| {
    let regex = r"^ *(?:(?<setlocation>set +location)|(?<step>step)|(?<newtest>new +test)|(?<requirement>requirement)|(?<abstract>abstract))(?<label>.*)";
    RegexBuilder::new(regex)
        .case_insensitive(true)
        .build()
        .expect("Failed to compile known commands regex.")
});

impl<'a> RoswaalTestSyntaxCommand<'a> {
    fn new(name: &'a str, description: &'a str) -> Self {
        let captures = match KNOWN_COMMANDS_REGEX.captures(name) {
            Some(c) => c,
            None => return RoswaalTestSyntaxCommand::UnknownCommand,
        };
        let label = captures
            .name("label")
            .map(|lmatch| lmatch.as_str())
            .unwrap_or("")
            .trim();
        if captures.name("setlocation").is_some() {
            return RoswaalTestSyntaxCommand::SetLocation {
                parse_result: RoswaalLocationName::from_str(description.trim()),
            };
        } else if captures.name("step").is_some() {
            return RoswaalTestSyntaxCommand::Step { label };
        } else if captures.name("newtest").is_some() {
            return RoswaalTestSyntaxCommand::NewTest;
        } else if captures.name("requirement").is_some() {
            return RoswaalTestSyntaxCommand::Requirement { label };
        } else {
            return RoswaalTestSyntaxCommand::Abstract;
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
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RoswaalTestSyntax<'a> {
    source_code: &'a str,
}

impl<'a> RoswaalTestSyntax<'a> {
    /// Returns an iterator of syntax tokens for each line in the source code.
    pub fn lines(&self) -> impl Iterator<Item = RoswaalTestSyntaxLine> {
        self.source_code
            .lines()
            .enumerate()
            .filter_map(|(i, line)| {
                RoswaalTestSyntaxLineContent::from(line).map(|content| RoswaalTestSyntaxLine {
                    line_number: (i + 1) as u32,
                    content,
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
        };
    }

    pub fn source_code(&self) -> &str {
        &self.source_code
    }

    /// Returns this syntax as a markdown code block.
    ///
    /// If this syntax is blank, then an indicator for empty source code will be returned.
    pub fn markdown_code_block(&self) -> String {
        if self.source_code().is_empty() {
            "```(Empty Source Code)```".to_string()
        } else {
            format!("```{}```", self.source_code())
        }
    }
}

impl<'a> From<&'a str> for RoswaalTestSyntax<'a> {
    fn from(source_code: &'a str) -> Self {
        Self { source_code }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct RoswaalTestSyntaxLine<'a> {
    line_number: u32,
    content: RoswaalTestSyntaxLineContent<'a>,
}

impl<'a> RoswaalTestSyntaxLine<'a> {
    pub fn line_number(&self) -> u32 {
        self.line_number
    }
    pub fn content(&self) -> &RoswaalTestSyntaxLineContent<'a> {
        &self.content
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum RoswaalTestSyntaxLineContent<'a> {
    Unknown(&'a str),
    Command {
        name: &'a str,
        description: &'a str,
        command: RoswaalTestSyntaxCommand<'a>,
    },
}

impl<'a> RoswaalTestSyntaxLineContent<'a> {
    pub fn from(line: &'a str) -> Option<Self> {
        let (name, description) = match line.split_once(":") {
            Some(split) => split,
            None => {
                return if line.is_empty() {
                    None
                } else {
                    Some(Self::Unknown(line))
                }
            }
        };
        let content = Self::Command {
            name,
            description: description.trim(),
            command: RoswaalTestSyntaxCommand::new(name, description),
        };
        Some(content)
    }
}

static EXTRACT_REGEX: Lazy<Regex> = Lazy::new(|| {
    let regex = r"(?s)```(?<test>.*?)```";
    RegexBuilder::new(regex)
        .build()
        .expect("Failed to compile extract tests syntax regex.")
});

/// Extracts a vector of `RoswaalTestSyntax` from a block of multiline text.
///
/// Each test should be placed between 2 "```\n" sequences in the string.
pub fn extract_tests_syntax<'a>(text: &'a str) -> Vec<RoswaalTestSyntax<'a>> {
    EXTRACT_REGEX
        .captures_iter(text)
        .filter_map(|captures| {
            captures
                .name("test")
                .map(|m| RoswaalTestSyntax::from(m.as_str().trim()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod extract_tests {
        use crate::language::ast::{extract_tests_syntax, RoswaalTestSyntax};

        #[test]
        fn test_no_syntax_from_empty_str() {
            assert_eq!(extract_tests_syntax(""), vec![])
        }

        #[test]
        fn test_syntax_from_markdown_str() {
            let markdown = "
```
New test: Test
Step 1: Get parsed
Requirement 1: Cool
```

Hello world, I am some random comment between tests, not an actual test...

```New Test: test 2
Step 1: Nice
Requirement 1: Awesome```

More BS **here** at the bottom.
";
            assert_eq!(
                extract_tests_syntax(markdown),
                vec![
                    RoswaalTestSyntax::from(
                        "\
New test: Test
Step 1: Get parsed
Requirement 1: Cool"
                    ),
                    RoswaalTestSyntax::from(
                        "\
New Test: test 2
Step 1: Nice
Requirement 1: Awesome"
                    )
                ]
            )
        }
    }

    #[cfg(test)]
    mod token_tests {
        use std::str::FromStr;

        use crate::location::name::{RoswaalLocationName, RoswaalLocationNameParsingError};

        use super::*;

        #[test]
        fn test_from_string_returns_empty_line_for_empty_string() {
            assert_eq!(RoswaalTestSyntaxLineContent::from(""), None)
        }

        #[test]
        fn test_from_string_returns_unknown_for_jibberish() {
            let source = "Sorry King Kai! I didn't know where else to bring him!";
            let content = RoswaalTestSyntaxLineContent::from(source);
            assert_eq!(content, Some(RoswaalTestSyntaxLineContent::Unknown(source)))
        }

        #[test]
        fn test_from_string_returns_step_for_simple_step_commands() {
            fn assert_step_description(line: &str, name: &str, description: &str, label: &str) {
                assert_command(
                    line,
                    name,
                    description,
                    RoswaalTestSyntaxCommand::Step { label },
                )
            }
            assert_step_description(
                "Step 1: Hello world this is a test",
                "Step 1",
                "Hello world this is a test",
                "1",
            );
            assert_step_description("Step: i am batman", "Step", "i am batman", "");
            assert_step_description("step 1: Hello", "step 1", "Hello", "1");
            assert_step_description("   step    1   : Hello", "   step    1   ", "Hello", "1");
            assert_step_description(
                "step one: Hello world this is a test",
                "step one",
                "Hello world this is a test",
                "one",
            );
            assert_step_description("step:", "step", "", "")
        }

        #[test]
        fn test_from_string_returns_set_location_for_set_location_commands() {
            fn assert_set_location(line: &str, command_name: &str, name: &str) {
                let parse_result = RoswaalLocationName::from_str(name);
                let command = RoswaalTestSyntaxCommand::SetLocation {
                    parse_result: parse_result.clone(),
                };
                assert_command(line, command_name, name, command);
                assert_eq!(name, parse_result.unwrap().raw_name())
            }

            fn assert_set_location_with_error(line: &str, error: RoswaalLocationNameParsingError) {
                let content = RoswaalTestSyntaxLineContent::from(line);
                let result = match content {
                    Some(RoswaalTestSyntaxLineContent::Command {
                        name: _,
                        description: _,
                        command,
                    }) => Some(command),
                    _ => None,
                }
                .and_then(|t| match t {
                    RoswaalTestSyntaxCommand::SetLocation { parse_result } => Some(parse_result),
                    _ => None,
                })
                .unwrap();
                assert_eq!(result, Err(error))
            }

            assert_set_location("set location: Apple", "set location", "Apple");
            assert_set_location("Set Location: Houston", "Set Location", "Houston");
            assert_set_location("Set      location: A", "Set      location", "A");
            assert_set_location("Set      location      : A", "Set      location      ", "A");
            assert_set_location(
                "   Set      location      : A",
                "   Set      location      ",
                "A",
            );
            assert_set_location_with_error("set location:", RoswaalLocationNameParsingError::Empty)
        }

        #[test]
        fn test_from_string_returns_unknown_command_for_random_commands() {
            fn assert_unknown_command(line: &str, name: &str, description: &str) {
                assert_command(
                    line,
                    name,
                    description,
                    RoswaalTestSyntaxCommand::UnknownCommand,
                )
            }

            assert_unknown_command("dkjhkjdh: hello", "dkjhkjdh", "hello");
            assert_unknown_command("dkjh kjdh: hello ", "dkjh kjdh", "hello")
        }

        #[test]
        fn test_from_string_returns_new_test_for_new_test_command() {
            fn assert_new_test(line: &str, name: &str, test_name: &str) {
                assert_command(line, name, test_name, RoswaalTestSyntaxCommand::NewTest)
            }

            assert_new_test("New Test: Hello world", "New Test", "Hello world");
            assert_new_test("new test: test", "new test", "test");
            assert_new_test(" new    tESt    : weird  ", " new    tESt    ", "weird")
        }

        #[test]
        fn test_from_string_returns_requirement_for_requirement_command() {
            fn assert_requirement(line: &str, name: &str, description: &str, label: &str) {
                assert_command(
                    line,
                    name,
                    description,
                    RoswaalTestSyntaxCommand::Requirement { label },
                )
            }

            assert_requirement(
                "Requirement 1: Hello world",
                "Requirement 1",
                "Hello world",
                "1",
            );
            assert_requirement("requirement: test", "requirement", "test", "");
            assert_requirement(
                " requirement   4: weird  ",
                " requirement   4",
                "weird",
                "4",
            );
            assert_requirement(
                "requirement one: weird  ",
                "requirement one",
                "weird",
                "one",
            )
        }

        #[test]
        fn test_from_string_returns_abstract_for_abstract_command() {
            fn assert_abstract(line: &str, name: &str, description: &str) {
                assert_command(line, name, description, RoswaalTestSyntaxCommand::Abstract)
            }

            assert_abstract("Abstract 1: Hello world", "Abstract 1", "Hello world");
            assert_abstract("abstract: test", "abstract", "test");
            assert_abstract(" abstract   4: weird  ", " abstract   4", "weird")
        }

        fn assert_command(
            line: &str,
            name: &str,
            description: &str,
            command: RoswaalTestSyntaxCommand,
        ) {
            let expected_content = RoswaalTestSyntaxLineContent::Command {
                name,
                description,
                command,
            };
            assert_eq!(
                RoswaalTestSyntaxLineContent::from(line),
                Some(expected_content)
            )
        }
    }

    #[cfg(test)]
    mod syntax_tests {
        use std::str::FromStr;

        use crate::{
            language::ast::{RoswaalTestSyntaxLine, RoswaalTestSyntaxLineContent},
            location::name::RoswaalLocationName,
        };

        use super::{RoswaalTestSyntax, RoswaalTestSyntaxCommand};

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
            let tokens = syntax.lines().collect::<Vec<RoswaalTestSyntaxLine>>();
            assert_eq!(
                tokens,
                vec!(
                    RoswaalTestSyntaxLine {
                        line_number: 1,
                        content: RoswaalTestSyntaxLineContent::Command {
                            name: "New Test",
                            description: "Something cool",
                            command: RoswaalTestSyntaxCommand::NewTest
                        }
                    },
                    RoswaalTestSyntaxLine {
                        line_number: 2,
                        content: RoswaalTestSyntaxLineContent::Command {
                            name: "Step 1",
                            description: "Write a step",
                            command: RoswaalTestSyntaxCommand::Step { label: "1" }
                        }
                    },
                    RoswaalTestSyntaxLine {
                        line_number: 3,
                        content: RoswaalTestSyntaxLineContent::Command {
                            name: "Step 2",
                            description: "Another step",
                            command: RoswaalTestSyntaxCommand::Step { label: "2" }
                        }
                    },
                    RoswaalTestSyntaxLine {
                        line_number: 4,
                        content: RoswaalTestSyntaxLineContent::Command {
                            name: "Set Location",
                            description: "Europe",
                            command: RoswaalTestSyntaxCommand::SetLocation {
                                parse_result: RoswaalLocationName::from_str("Europe")
                            }
                        }
                    },
                    RoswaalTestSyntaxLine {
                        line_number: 5,
                        content: RoswaalTestSyntaxLineContent::Command {
                            name: "Big",
                            description: "chungus",
                            command: RoswaalTestSyntaxCommand::UnknownCommand
                        }
                    },
                    RoswaalTestSyntaxLine {
                        line_number: 7,
                        content: RoswaalTestSyntaxLineContent::Command {
                            name: "Requirement 1",
                            description: "Do the thing",
                            command: RoswaalTestSyntaxCommand::Requirement { label: "1" }
                        }
                    },
                    RoswaalTestSyntaxLine {
                        line_number: 8,
                        content: RoswaalTestSyntaxLineContent::Command {
                            name: "Requirement 2",
                            description: "Do the other thing",
                            command: RoswaalTestSyntaxCommand::Requirement { label: "2" }
                        }
                    }
                )
            )
        }
    }
}
