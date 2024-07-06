use anyhow::Result;

use crate::{location::location::RoswaalStringLocations, utils::sqlite::RoswaalSqlite, with_transaction};

#[derive(Debug, PartialEq)]
pub enum AddLocationsStatus {
    Success(RoswaalStringLocations),
    NoLocationsAdded
}

impl AddLocationsStatus {
    pub async fn from_adding_locations(
        locations_str: &str,
        sqlite: &RoswaalSqlite
    ) -> Result<Self> {
        if locations_str.is_empty() {
            return Ok(Self::NoLocationsAdded)
        }
        let string_locations = RoswaalStringLocations::from_roswaal_locations_str(locations_str);
        let mut transaction = sqlite.transaction().await?;
        with_transaction!(transaction, async {
            transaction.save_locations(&string_locations.locations()).await?;
            Ok(Self::Success(string_locations))
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{location::location::RoswaalStringLocations, operations::add_locations::AddLocationsStatus, utils::sqlite::RoswaalSqlite};

    #[tokio::test]
    async fn test_success_when_adding_locations_smoothly() {
        let str = "Test, 50.0, 50.0";
        let sqlite = RoswaalSqlite::in_memory().await.unwrap();
        let result = AddLocationsStatus::from_adding_locations(str, &sqlite).await;
        let str_locations = RoswaalStringLocations::from_roswaal_locations_str(str);
        assert_eq!(result.ok(), Some(AddLocationsStatus::Success(str_locations)))
    }

    #[tokio::test]
    async fn test_success_mixes_proper_and_invalid_locations() {
        let str = "Test, 50.0, 50.0\n29879";
        let sqlite = RoswaalSqlite::in_memory().await.unwrap();
        let result = AddLocationsStatus::from_adding_locations(str, &sqlite).await;
        let str_locations = RoswaalStringLocations::from_roswaal_locations_str(str);
        assert_eq!(result.ok(), Some(AddLocationsStatus::Success(str_locations)))
    }

    #[tokio::test]
    async fn test_no_locations_added_when_empty_vector() {
        let sqlite = RoswaalSqlite::in_memory().await.unwrap();
        let result = AddLocationsStatus::from_adding_locations("", &sqlite).await;
        assert_eq!(result.ok(), Some(AddLocationsStatus::NoLocationsAdded))
    }
}
