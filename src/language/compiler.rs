#[derive(Debug, PartialEq, Eq)]
pub struct RoswaalCompilationError {
    pub line_number: u32,
    pub code: RoswaalCompilationErrorCode
}

#[derive(Debug, PartialEq, Eq)]
pub enum RoswaalCompilationErrorCode {
    NoTestName,
    NoTestSteps,
    InvalidStepName(String)
}
