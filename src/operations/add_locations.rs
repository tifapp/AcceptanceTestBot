use anyhow::Result;
use crate::{location::location::{FromRoswaalLocationsStr, RoswaalLocation, RoswaalLocationStringError}, utils::sqlite::RoswaalSqlite, with_transaction};

#[derive(Debug, PartialEq, Eq)]
pub enum AddLocationsResult {
    Success,
    NoLocationsAdded
}

impl AddLocationsResult {
    pub async fn from_adding_locations(
        locations_str: &str,
        sqlite: &RoswaalSqlite
    ) -> Result<Self> {
        if locations_str.is_empty() {
            return Ok(Self::NoLocationsAdded)
        }
        let mut locations_vec = Vec::<RoswaalLocation>::new();
        let parsed_locations = Vec::<Result<RoswaalLocation, RoswaalLocationStringError>>
            ::from_roswaal_locations_str(locations_str);
        for result in parsed_locations {
            if let Ok(location) = result {
                locations_vec.push(location)
            }
        }
        let mut transaction = sqlite.transaction().await?;
        with_transaction!(transaction, async {
            transaction.save_locations(&locations_vec).await?;
            Ok(Self::Success)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{operations::add_locations::AddLocationsResult, utils::sqlite::RoswaalSqlite};

    #[tokio::test]
    async fn test_success_when_adding_locations_smoothly() {
        let sqlite = RoswaalSqlite::in_memory().await.unwrap();
        let result = AddLocationsResult::from_adding_locations("Test, 50.0, 50.0", &sqlite).await;
        assert_eq!(result.ok(), Some(AddLocationsResult::Success))
    }

    #[tokio::test]
    async fn test_no_locations_added_when_empty_vector() {
        let sqlite = RoswaalSqlite::in_memory().await.unwrap();
        let result = AddLocationsResult::from_adding_locations("", &sqlite).await;
        assert_eq!(result.ok(), Some(AddLocationsResult::NoLocationsAdded))
    }
}
