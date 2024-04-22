/// A token of roswaal test syntax.
///
/// Each token represents a line of source code. See `RoswaalTestSyntax`.
pub enum RoswaalTestSyntaxToken {
    /// A line denoting a "Step" command without its matching "Requirement"
    /// command.
    Step { description: String },
    /// A line denoting the "New Test" command.
    NewTest { name: String },
    /// A line denoting the "SetLocation" command.
    SetLocation { unchecked_name: String },
    /// A line denoting the "Requirement" command that is to be paired with a
    /// respective step command.
    Requirement { description: String },
    /// A line which has proper command syntax, but the command is not known.
    UnknownCommand { name: String, description: String },
    /// A line which does not follow traditional command syntax.
    Unknown { source: String },
    /// An empty line.
    EmptyLine
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
}

impl From<&str> for RoswaalTestSyntax {
    fn from(source_code: &str) -> Self {
        Self { source_code: source_code.to_string() }
    }
}
