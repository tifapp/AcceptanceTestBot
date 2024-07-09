use serde::Deserialize;

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct RoswaalTestProgress {
    test_name: String,
    results: Vec<RoswaalTestProgressResult>,
    error: Option<RoswaalTestProgressErrorDescription>
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct RoswaalTestProgressErrorDescription {
    message: String,
    stack_trace: String
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct RoswaalTestProgressResult {
    did_pass: bool
}
