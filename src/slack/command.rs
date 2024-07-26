use strum_macros::EnumString;

/// The slack slash commands that this tool must respond to.
#[derive(Debug, PartialEq, Eq, EnumString)]
pub enum RoswaalSlackCommand {
    #[strum(serialize="/view-tests")]
    ViewTests,
    #[strum(serialize="/add-tests")]
    AddTests,
    #[strum(serialize="/remove-tests")]
    RemoveTests,
    #[strum(serialize="/view-locations")]
    ViewLocations,
    #[strum(serialize="/add-locations")]
    AddLocations
}
