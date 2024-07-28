use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub struct TestError;

impl Display for TestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TestError")
    }
}

impl Error for TestError {}
