use crate::location::name::RoswaalLocationName;

#[derive(Debug, PartialEq, Eq)]
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
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RoswaalTestCommand {
    Step { name: String, requirement: String },
    SetLocation { location_name: RoswaalLocationName }
}
