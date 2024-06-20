use std::str::FromStr;

use super::{coordinate::LocationCoordinate2D, name::{RoswaalLocationName, RoswaalLocationNameParsingError, RoswaalLocationParsingResult}};

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

    pub(super) fn new_without_validation(name: &str, latitude: f32, longitude: f32) -> Self {
        let name = RoswaalLocationName { raw_value: name.to_string() };
        let coordinate = LocationCoordinate2D { latitude, longitude };
        Self::new(name, coordinate)
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

#[derive(Debug, PartialEq, Eq)]
pub enum RoswaalLocationStringError {
    InvalidName(String, RoswaalLocationNameParsingError),
    InvalidCoordinate { name: String }
}

impl FromStr for RoswaalLocation {
    type Err = RoswaalLocationStringError;

    fn from_str(l: &str) -> Result<Self, Self::Err> {
        let splits = l.splitn(3, ",").collect::<Vec<&str>>();
        let raw_name = splits[0];
        let name = RoswaalLocationName::from_str(raw_name);
        if let Err(err) = name {
            return Err(Self::Err::InvalidName(raw_name.to_string(), err))
        }
        if splits.len() < 3 {
            return Err(Self::Err::InvalidCoordinate { name: raw_name.to_string() })
        }
        let latitude = splits[1].trim().parse::<f32>();
        let longitude = splits[2].trim().parse::<f32>();
        match (name, latitude, longitude) {
            (Ok(name), Ok(lat), Ok(lng)) => {
                if let Some(coordinate) = LocationCoordinate2D::try_new(lat, lng) {
                    Ok(RoswaalLocation::new(name, coordinate))
                } else {
                    Err(Self::Err::InvalidCoordinate { name: raw_name.to_string() })
                }
            },
            _ => Err(Self::Err::InvalidCoordinate { name: raw_name.to_string() })
        }
    }
}

pub trait FromRoswaalStr: Sized {
    fn from_roswaal_str(str: &str) -> Self;
}

impl FromRoswaalStr for Vec<Result<RoswaalLocation, RoswaalLocationStringError>> {
    fn from_roswaal_str(str: &str) -> Self {
        str.lines()
            .filter(|l| !l.trim().is_empty())
            .map(RoswaalLocation::from_str)
            .collect::<Self>()
    }
}

#[cfg(test)]
mod tests {
    mod from_str_tests {
        use crate::location::{location::{FromRoswaalStr, RoswaalLocation, RoswaalLocationStringError}, name::RoswaalLocationNameParsingError};

        #[test]
        fn test_returns_empty_vector_when_empty_string() {
            let locations = Vec::<Result<RoswaalLocation, RoswaalLocationStringError>>::from_roswaal_str("");
            assert_eq!(locations, vec![])
        }

        #[test]
        fn test_returns_locations_from_multiline_string() {
            let str = "
Antarctica, 50.0, 50.0
New York, 45.0, 45.0
San Francisco, 12.298739, 122.2989379

Test 4, 0.0, 0.0
   Whitespace   ,      2.198   ,         3.1415
                ";
            let locations = Vec::<Result<RoswaalLocation, RoswaalLocationStringError>>::from_roswaal_str(str);
            let expected_locations = vec![
                Ok(RoswaalLocation::new_without_validation("Antarctica", 50.0, 50.0)),
                Ok(RoswaalLocation::new_without_validation("New York", 45.0, 45.0)),
                Ok(RoswaalLocation::new_without_validation("San Francisco", 12.298739, 122.2989379)),
                Ok(RoswaalLocation::new_without_validation("Test 4", 0.0, 0.0)),
                Ok(RoswaalLocation::new_without_validation("   Whitespace   ", 2.198, 3.1415)),
            ];
            assert_eq!(locations, expected_locations)
        }

        #[test]
        fn test_returns_errors_with_locations_from_multiline_string() {
            let str = "
Antarctica, 50.0, 50.0
New York
12.298739, 122.2989379
Test 4, hello, 0.0
Test 5, -80.0, world
Test 6, -400.0, 400
                ";
            let locations = Vec::<Result<RoswaalLocation, RoswaalLocationStringError>>::from_roswaal_str(str);
            let expected_locations = vec![
                Ok(RoswaalLocation::new_without_validation("Antarctica", 50.0, 50.0)),
                Err(RoswaalLocationStringError::InvalidCoordinate { name: "New York".to_string() }),
                Err(RoswaalLocationStringError::InvalidName(
                    "12.298739".to_string(),
                    RoswaalLocationNameParsingError::InvalidFormat
                )),
                Err(RoswaalLocationStringError::InvalidCoordinate { name: "Test 4".to_string() }),
                Err(RoswaalLocationStringError::InvalidCoordinate { name: "Test 5".to_string() }),
                Err(RoswaalLocationStringError::InvalidCoordinate { name: "Test 6".to_string() })
            ];
            assert_eq!(locations, expected_locations)
        }
    }
}
