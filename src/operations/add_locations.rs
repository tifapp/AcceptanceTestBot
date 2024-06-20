use anyhow::Result;
use crate::{location::location::RoswaalLocation, utils::sqlite::RoswaalSqlite, with_transaction};

#[derive(Debug, PartialEq, Eq)]
pub enum AddLocationsResult {
    Success,
    NoLocationsAdded
}

impl AddLocationsResult {
    pub async fn from_adding_locations(
        locations: &Vec<RoswaalLocation>,
        sqlite: &RoswaalSqlite
    ) -> Result<Self> {
        if locations.is_empty() {
            return Ok(Self::NoLocationsAdded)
        }
        let mut transaction = sqlite.transaction().await?;
        with_transaction!(transaction, async {
            transaction.save_locations(locations).await?;
            Ok(Self::Success)
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{location::{coordinate::LocationCoordinate2D, location::RoswaalLocation, name::RoswaalLocationName}, operations::add_locations::AddLocationsResult, utils::sqlite::RoswaalSqlite};

    #[tokio::test]
    async fn test_success_when_adding_locations_smoothly() {
        let sqlite = RoswaalSqlite::in_memory().await.unwrap();
        let locations = vec![
            RoswaalLocation::new(
                RoswaalLocationName::from_str("Test").unwrap(),
                LocationCoordinate2D::try_new(50.0, 50.0).unwrap()
            )
        ];
        let result = AddLocationsResult::from_adding_locations(&locations, &sqlite).await;
        assert_eq!(result.ok(), Some(AddLocationsResult::Success))
    }

    #[tokio::test]
    async fn test_no_locations_added_when_empty_vector() {
        let sqlite = RoswaalSqlite::in_memory().await.unwrap();
        let result = AddLocationsResult::from_adding_locations(&vec![], &sqlite).await;
        assert_eq!(result.ok(), Some(AddLocationsResult::NoLocationsAdded))
    }
}
