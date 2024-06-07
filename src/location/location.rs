use super::{coordinate::LocationCoordinate2D, name::RoswaalLocationName};

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
