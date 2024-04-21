#[derive(Debug, PartialEq, Eq)]
pub struct RoswaalCompilationError {
    pub line_number: u32,
    pub code: RoswaalCompilationErrorCode
}

#[derive(Debug, PartialEq, Eq)]
pub enum RoswaalCompilationErrorCode {
    NoTestName,
    NoTestSteps,
    NoStepDescription { step_name: String },
    NoLocationSpecified,
    InvalidCommandName(String)
}
