use serde::{Deserialize, Serialize};

use crate::{location::name::RoswaalLocationName, utils::string::ToAsciiKebabCase};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RoswaalTest {
    name: String,
    description: Option<String>,
    commands: Vec<RoswaalTestCommand>
}

impl RoswaalTest {
    pub fn new(
        name: String,
        description: Option<String>,
        commands: Vec<RoswaalTestCommand>
    ) -> Self {
        Self { name, description, commands }
    }
}

impl RoswaalTest {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn commands(&self) -> &Vec<RoswaalTestCommand> {
        &self.commands
    }

    pub fn description(&self) -> Option<&String> {
        self.description.as_ref()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum RoswaalTestCommand {
    Step { name: String, requirement: String },
    SetLocation { location_name: RoswaalLocationName }
}
