use super::location::RoswaalLocationName;

#[derive(Debug, PartialEq, Eq)]
pub struct RoswaalTest {
    name: String,
    commands: Vec<RoswaalTestCommand>
}

impl RoswaalTest {
    pub fn new(name: String, commands: Vec<RoswaalTestCommand>) -> Self {
        Self { name, commands }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum RoswaalTestCommand {
    Step { name: String, requirement: String },
    SetLocation { location_name: RoswaalLocationName }
}
