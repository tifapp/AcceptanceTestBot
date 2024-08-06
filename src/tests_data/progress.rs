use serde::Deserialize;

use super::ordinal::RoswaalTestCommandOrdinal;

/// The progress of a test case.
///
/// Each test runs its commands sequentially, and reports a failure on the ordinal of the command.
/// Note that the zero ordinal denotes the before launch command, which every test implicity has.
#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoswaalTestProgressUpload {
    test_name: String,
    command_failure_ordinal: Option<RoswaalTestCommandOrdinal>,
    error: Option<RoswaalTestProgressUploadErrorDescription>,
}

impl RoswaalTestProgressUpload {
    pub fn new(
        test_name: String,
        command_failure_ordinal: Option<RoswaalTestCommandOrdinal>,
        error: Option<RoswaalTestProgressUploadErrorDescription>,
    ) -> Self {
        Self {
            test_name,
            command_failure_ordinal,
            error,
        }
    }
}

impl RoswaalTestProgressUpload {
    pub fn command_failure_ordinal(&self) -> Option<RoswaalTestCommandOrdinal> {
        self.command_failure_ordinal
    }

    pub fn test_name(&self) -> &str {
        &self.test_name
    }

    pub fn error_message(&self) -> Option<&String> {
        self.error.as_ref().map(|e| &e.message)
    }

    pub fn error_stack_trace(&self) -> Option<&String> {
        self.error.as_ref().map(|e| &e.stack_trace)
    }
}

#[derive(Debug, PartialEq, Eq, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RoswaalTestProgressUploadErrorDescription {
    message: String,
    stack_trace: String,
}

impl RoswaalTestProgressUploadErrorDescription {
    pub fn new(message: String, stack_trace: String) -> Self {
        Self {
            message,
            stack_trace,
        }
    }
}
