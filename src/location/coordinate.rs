/// A latitude-longitude coordinate that assumes a spherical-earth.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct LocationCoordinate2D {
    pub(super) latitude: f32,
    pub(super) longitude: f32,
}

impl LocationCoordinate2D {
    /// Attempts to create a LocationCoordinate2D from a specified latitude and longitudal
    /// coordinate.
    ///
    /// The latitude must be in \[-90, 90\], and longitude in \[-180, 180\].
    pub fn try_new(latitude: f32, longitude: f32) -> Option<Self> {
        if !(-90.0..90.0).contains(&latitude) || !(-180.0..180.0).contains(&longitude) {
            None
        } else {
            Some(Self {
                latitude,
                longitude,
            })
        }
    }
}

impl LocationCoordinate2D {
    pub fn latitude(&self) -> f32 {
        self.latitude
    }

    pub fn longitude(&self) -> f32 {
        self.longitude
    }
}

#[cfg(test)]
mod test {
    use crate::location::coordinate::LocationCoordinate2D;

    #[test]
    fn test_coordinate_creation() {
        assert_eq!(
            LocationCoordinate2D::try_new(-100.79832794, 67.290890),
            None
        );
        assert_eq!(LocationCoordinate2D::try_new(43.29872, 200.2979398), None);
        assert_eq!(LocationCoordinate2D::try_new(100.79832794, 67.290890), None);
        assert_eq!(LocationCoordinate2D::try_new(43.29872, -200.2979398), None);
        assert_eq!(
            LocationCoordinate2D::try_new(32.209820, -120.289739),
            Some(LocationCoordinate2D {
                latitude: 32.209820,
                longitude: -120.289739
            })
        )
    }
}
