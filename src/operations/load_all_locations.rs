use anyhow::Result;
use crate::{location::location::RoswaalLocation, utils::sqlite::RoswaalSqlite, with_transaction};

#[derive(Debug, PartialEq)]
pub enum LoadAllLocationsResult {
    Success(Vec<RoswaalLocation>),
    NoLocations
}

impl LoadAllLocationsResult {
    pub async fn from_stored_locations(
        sqlite: &RoswaalSqlite
    ) -> Result<Self> {
        let mut transaction = sqlite.transaction().await?;
        with_transaction!(transaction, async {
            transaction.locations_in_alphabetical_order().await.map(|locations| {
                if locations.is_empty() { Self::NoLocations } else { Self::Success(locations) }
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{location::{coordinate::LocationCoordinate2D, location::RoswaalLocation, name::RoswaalLocationName}, operations::{add_locations::AddLocationsResult, load_all_locations::LoadAllLocationsResult}, utils::sqlite::RoswaalSqlite};

    #[tokio::test]
    async fn test_returns_no_locations_when_no_saved_locations() {
        let sqlite = RoswaalSqlite::in_memory().await.unwrap();
        let result = LoadAllLocationsResult::from_stored_locations(&sqlite).await.unwrap();
        assert_eq!(result, LoadAllLocationsResult::NoLocations)
    }

    #[tokio::test]
    async fn test_returns_locations_from_add_operation() {
        let sqlite = RoswaalSqlite::in_memory().await.unwrap();
        let locations = vec![
            RoswaalLocation::new(
                RoswaalLocationName::from_str("Test 1").unwrap(),
                LocationCoordinate2D::try_new(50.0, 50.0).unwrap()
            ),
            RoswaalLocation::new(
                RoswaalLocationName::from_str("Test 2").unwrap(),
                LocationCoordinate2D::try_new(-5.0, 5.0).unwrap()
            )
        ];
        _ = AddLocationsResult::from_adding_locations(&locations, &sqlite).await.unwrap();
        let result = LoadAllLocationsResult::from_stored_locations(&sqlite).await.unwrap();
        assert_eq!(result, LoadAllLocationsResult::Success(locations))
    }
}
