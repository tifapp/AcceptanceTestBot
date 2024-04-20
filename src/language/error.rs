use std::{error::Error, fmt};

#[derive(Debug, PartialEq, Eq)]
pub enum RoswaalCompilationError {
    NoTestName,
    NoTestSteps,
    InvalidStepName(String)
}

impl fmt::Display for RoswaalCompilationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Compilation Error: {}", self)
    }
}

impl Error for RoswaalCompilationError {}
