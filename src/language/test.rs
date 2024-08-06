use serde::{Deserialize, Serialize};

use crate::location::name::RoswaalLocationName;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RoswaalCompiledTest {
    name: String,
    description: Option<String>,
    commands: Vec<RoswaalCompiledTestCommand>,
}

impl RoswaalCompiledTest {
    pub fn new(
        name: String,
        description: Option<String>,
        commands: Vec<RoswaalCompiledTestCommand>,
    ) -> Self {
        Self {
            name,
            description,
            commands,
        }
    }
}

impl RoswaalCompiledTest {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn commands(&self) -> &Vec<RoswaalCompiledTestCommand> {
        &self.commands
    }

    pub fn description(&self) -> Option<&String> {
        self.description.as_ref()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum RoswaalCompiledTestCommand {
    Step {
        label: String,
        name: String,
        requirement: String,
    },
    SetLocation {
        location_name: RoswaalLocationName,
    },
}
