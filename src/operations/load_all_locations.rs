use anyhow::Result;
use crate::{location::location::RoswaalLocation, utils::sqlite::RoswaalSqlite, with_transaction};

#[derive(Debug, PartialEq)]
pub enum LoadAllLocationsStatus {
    Success(Vec<RoswaalLocation>),
    NoLocations
}

impl LoadAllLocationsStatus {
    pub async fn from_stored_locations(
        sqlite: &RoswaalSqlite
    ) -> Result<Self> {
        let mut transaction = sqlite.transaction().await?;
        with_transaction!(transaction, async {
            transaction.locations_in_alphabetical_order().await.map(|locations| {
                if locations.is_empty() {
                    Self::NoLocations
                } else {
                    Self::Success(locations.iter().map(|l| l.location().clone()).collect())
                }
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{location::location::RoswaalLocation, operations::{add_locations::AddLocationsStatus, load_all_locations::LoadAllLocationsStatus}, utils::sqlite::RoswaalSqlite};

    #[tokio::test]
    async fn test_returns_no_locations_when_no_saved_locations() {
        let sqlite = RoswaalSqlite::in_memory().await.unwrap();
        let status = LoadAllLocationsStatus::from_stored_locations(&sqlite).await.unwrap();
        assert_eq!(status, LoadAllLocationsStatus::NoLocations)
    }

    #[tokio::test]
    async fn test_returns_locations_from_add_operation() {
        let sqlite = RoswaalSqlite::in_memory().await.unwrap();
        let locations = vec![
            RoswaalLocation::new_without_validation("Test 1", 50.0, 50.0),
            RoswaalLocation::new_without_validation("Test 2", -5.0, 5.0)
        ];
        let locations_str = "
Test 1, 50.0, 50.0
Invalid
Test 2, -5.0, 5.0
            ";
        _ = AddLocationsStatus::from_adding_locations(&locations_str, &sqlite).await.unwrap();
        let status = LoadAllLocationsStatus::from_stored_locations(&sqlite).await.unwrap();
        assert_eq!(status, LoadAllLocationsStatus::Success(locations))
    }
}
