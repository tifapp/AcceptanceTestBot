use super::{coordinate::LocationCoordinate2D, name::RoswaalLocationName};

/// A location with a name and coordinate.
#[derive(Debug, PartialEq, Clone)]
pub struct RoswaalLocation {
    name: RoswaalLocationName,
    coordinate: LocationCoordinate2D
}

impl RoswaalLocation {
    pub fn new(name: RoswaalLocationName, coordinate: LocationCoordinate2D) -> Self {
        Self { name, coordinate }
    }
}

impl RoswaalLocation {
    pub fn name(&self) -> &RoswaalLocationName {
        &self.name
    }

    pub fn coordinate(&self) -> LocationCoordinate2D {
        self.coordinate
    }
}
