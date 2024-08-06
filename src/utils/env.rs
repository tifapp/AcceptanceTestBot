use std::env;

/// An enum representing the current environment that this tool is running in.
#[derive(Debug, PartialEq, Eq)]
pub enum RoswaalEnvironement {
    Dev,
    Prod,
}

impl RoswaalEnvironement {
    /// Returns the current environment based on the `ROSWAAL_ENV` environment variable.
    pub fn current() -> Self {
        if env::var("ROSWAAL_ENV").map(|e| e == "dev").is_ok() {
            Self::Dev
        } else {
            Self::Prod
        }
    }
}
