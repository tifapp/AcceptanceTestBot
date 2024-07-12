use anyhow::Result;
use sqlx::{query, query_as, FromRow, Sqlite};

use crate::{git::branch_name::{self, RoswaalOwnedGitBranchName}, utils::sqlite::{sqlite_repeat, RoswaalSqliteTransaction}};

use super::{location::RoswaalLocation, name::RoswaalLocationName};

#[derive(Debug, PartialEq)]
pub struct RoswaalStoredLocation {
    location: RoswaalLocation,
    unmerged_branch_name: Option<RoswaalOwnedGitBranchName>
}

impl RoswaalStoredLocation {
    pub fn location(&self) -> &RoswaalLocation {
        &self.location
    }
}

pub enum LoadLocationsFilter {
    All,
    MergedOnly
}

impl LoadLocationsFilter {
    fn full_select_statement(&self) -> &'static str {
        match self {
            Self::All => statements::SELECT_ALL_LOCATIONS,
            Self::MergedOnly => statements::SELECT_ALL_MERGED_LOCATIONS
        }
    }

    fn name_select_statement(&self) -> &'static str {
        match self {
            Self::All => statements::SELECT_ALL_LOCATION_NAMES,
            Self::MergedOnly => statements::SELECT_ALL_MERGED_LOCATION_NAMES
        }
    }
}

impl <'a> RoswaalSqliteTransaction <'a> {
    pub async fn merge_unmerged_locations(
        &mut self,
        branch_name: &RoswaalOwnedGitBranchName
    ) -> Result<()> {
        let sqlite_location_names = query_as::<Sqlite, SqliteLocationName>(
            statements::SELECT_UNMERGED_LOCATION_NAMES_WITH_BRANCH
        )
        .bind(branch_name)
        .fetch_all(self.connection())
        .await?;
        sqlite_repeat(statements::MERGE_UNMERGED_LOCATION, &sqlite_location_names)
            .bind_to_query(|q, sqlite_name| {
                Ok(q.bind(sqlite_name.name.clone()).bind(branch_name).bind(sqlite_name.name.clone()))
            })?
            .execute(self.connection()).await?;
        Ok(())
    }

    pub async fn save_locations(
        &mut self,
        locations: &Vec<RoswaalLocation>,
        branch_name: &RoswaalOwnedGitBranchName
    ) -> Result<()> {
        sqlite_repeat::<RoswaalLocation>(statements::INSERT_OR_REPLACE_LOCATION, locations)
            .bind_to_query(|q, location| {
                Ok(
                    q.bind(location.coordinate().latitude())
                        .bind(location.coordinate().longitude())
                        .bind(&location.name().raw_value)
                        .bind(branch_name)
                )
            })?
            .execute(self.connection()).await?;
        Ok(())
    }

    pub async fn locations_in_alphabetical_order(
        &mut self,
        filter: LoadLocationsFilter
    ) -> Result<Vec<RoswaalStoredLocation>> {
        let locations = query_as::<Sqlite, SqliteLocation>(filter.full_select_statement())
            .fetch_all(self.connection())
            .await?
            .iter()
            .map(|l| RoswaalStoredLocation {
                location: RoswaalLocation::new_without_validation(&l.name, l.latitude, l.longitude),
                unmerged_branch_name: l.unmerged_branch_name.clone()
            })
            .collect();
            Ok(locations)
    }

    pub async fn location_names_in_alphabetical_order(
        &mut self,
        filter: LoadLocationsFilter
    ) -> Result<Vec<RoswaalLocationName>> {
        let locations = query_as::<Sqlite, SqliteLocationName>(filter.name_select_statement())
            .fetch_all(self.connection())
            .await?
            .iter()
            .map(|n| RoswaalLocationName { raw_value: n.name.clone() })
            .collect();
        Ok(locations)
    }
}

mod statements {
    pub const INSERT_OR_REPLACE_LOCATION: &str = "
INSERT OR REPLACE INTO Locations (
    latitude,
    longitude,
    name,
    unmerged_branch_name
) VALUES (
    ?,
    ?,
    ?,
    ?
);";

    pub const MERGE_UNMERGED_LOCATION: &str = "
DELETE FROM Locations WHERE name = ? AND unmerged_branch_name IS NULL;
UPDATE Locations SET unmerged_branch_name = NULL WHERE unmerged_branch_name = ? AND name = ?;
";

    pub const SELECT_UNMERGED_LOCATION_NAMES_WITH_BRANCH: &str =
        "SELECT name FROM Locations WHERE unmerged_branch_name = ?;";

    pub const SELECT_ALL_LOCATION_NAMES: &str =
        "SELECT name FROM Locations ORDER BY name, latitude";

    pub const SELECT_ALL_MERGED_LOCATION_NAMES: &str =
        "SELECT name FROM Locations WHERE unmerged_branch_name IS NULL ORDER BY name, latitude;";

    pub const SELECT_ALL_LOCATIONS: &str =
        "SELECT * FROM Locations ORDER BY name, latitude";

    pub const SELECT_ALL_MERGED_LOCATIONS: &str =
        "SELECT * FROM Locations WHERE unmerged_branch_name IS NULL ORDER BY name, latitude;";
}

#[derive(FromRow, Debug)]
struct SqliteLocationName {
    name: String
}

#[derive(FromRow, Clone)]
struct SqliteLocation {
    latitude: f32,
    longitude: f32,
    name: String,
    unmerged_branch_name: Option<RoswaalOwnedGitBranchName>
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{git::branch_name::{self, RoswaalOwnedGitBranchName}, location::{location::RoswaalLocation, storage::RoswaalStoredLocation}, utils::sqlite::RoswaalSqlite};

    #[tokio::test]
    async fn test_add_and_load_locations_no_prior_locations() {
        let branch_name = RoswaalOwnedGitBranchName::new("test");
        let sqlite = RoswaalSqlite::in_memory().await.unwrap();
        let mut transaction = sqlite.transaction().await.unwrap();
        let locations = vec![
            RoswaalLocation::new_without_validation("Antarctica", 32.29873932, 122.3939839),
            RoswaalLocation::new_without_validation("New York", 45.0, 45.0)
        ];
        _ = transaction.save_locations(&locations, &branch_name).await;
        let saved_locations = transaction.locations_in_alphabetical_order(LoadLocationsFilter::All)
            .await.unwrap();
        let expected_locations = vec![
            RoswaalStoredLocation { location: locations[0].clone(), unmerged_branch_name: Some(branch_name.clone()) },
            RoswaalStoredLocation { location: locations[1].clone(), unmerged_branch_name: Some(branch_name.clone()) }
        ];
        assert_eq!(saved_locations, expected_locations)
    }

    #[tokio::test]
    async fn test_add_same_locations_on_same_branch_replaces_previous() {
        let branch_name = RoswaalOwnedGitBranchName::new("test");
        let sqlite = RoswaalSqlite::in_memory().await.unwrap();
        let mut transaction = sqlite.transaction().await.unwrap();
        let mut locations = vec![
            RoswaalLocation::new_without_validation("Antarctica", 32.29873932, 122.3939839)
        ];
        _ = transaction.save_locations(&locations, &branch_name).await;
        locations = vec![
            RoswaalLocation::new_without_validation("Antarctica", 45.20982, 78.209782972)
        ];
        _ = transaction.save_locations(&locations, &branch_name).await;
        let saved_locations = transaction.locations_in_alphabetical_order(LoadLocationsFilter::All)
            .await.unwrap();
        let expected_locations = vec![
            RoswaalStoredLocation { location: locations[0].clone(), unmerged_branch_name: Some(branch_name) }
        ];
        assert_eq!(saved_locations, expected_locations);
    }

    #[tokio::test]
    async fn test_add_and_load_locations_on_different_branches_adds_new_record() {
        let branch_name = RoswaalOwnedGitBranchName::new("test");
        let sqlite = RoswaalSqlite::in_memory().await.unwrap();
        let mut transaction = sqlite.transaction().await.unwrap();
        let mut locations = vec![
            RoswaalLocation::new_without_validation("Antarctica", 32.29873932, 122.3939839),
            RoswaalLocation::new_without_validation("New York", 45.0, 45.0)
        ];
        _ = transaction.save_locations(&locations, &branch_name).await;
        locations = vec![
            RoswaalLocation::new_without_validation("Antarctica", 50.0, 50.0),
            RoswaalLocation::new_without_validation("Oakland", 45.0, 45.0)
        ];
        let branch_name2 = RoswaalOwnedGitBranchName::new("test-2");
        _ = transaction.save_locations(&locations, &branch_name2).await;
        let saved_locations = transaction.locations_in_alphabetical_order(LoadLocationsFilter::All)
            .await.unwrap();
        let expected_locations = vec![
            RoswaalStoredLocation {
                location: RoswaalLocation::new_without_validation("Antarctica", 32.29873932, 122.3939839),
                unmerged_branch_name: Some(branch_name.clone())
            },
            RoswaalStoredLocation {
                location: RoswaalLocation::new_without_validation("Antarctica", 50.0, 50.0),
                unmerged_branch_name: Some(branch_name2.clone())
            },
            RoswaalStoredLocation {
                location: RoswaalLocation::new_without_validation("New York", 45.0, 45.0),
                unmerged_branch_name: Some(branch_name.clone())
            },
            RoswaalStoredLocation {
                location: RoswaalLocation::new_without_validation("Oakland", 45.0, 45.0),
                unmerged_branch_name: Some(branch_name2.clone())
            }
        ];
        assert_eq!(saved_locations, expected_locations)
    }

    #[tokio::test]
    async fn save_and_merge_locations_removes_unmerged_branch_name() {
        let branch_name = RoswaalOwnedGitBranchName::new("test");
        let sqlite = RoswaalSqlite::in_memory().await.unwrap();
        let mut transaction = sqlite.transaction().await.unwrap();
        let locations = vec![
            RoswaalLocation::new_without_validation("Antarctica", 32.29873932, 122.3939839),
            RoswaalLocation::new_without_validation("New York", 45.0, 45.0)
        ];
        transaction.save_locations(&locations, &branch_name).await.unwrap();
        transaction.merge_unmerged_locations(&branch_name).await.unwrap();
        let saved_locations = transaction.locations_in_alphabetical_order(LoadLocationsFilter::All)
            .await.unwrap();
        let expected_locations = vec![
            RoswaalStoredLocation { location: locations[0].clone(), unmerged_branch_name: None },
            RoswaalStoredLocation { location: locations[1].clone(), unmerged_branch_name: None }
        ];
        assert_eq!(saved_locations, expected_locations)
    }

    #[tokio::test]
    async fn save_and_merge_existing_location_overrides_previous_merged_location() {
        let sqlite = RoswaalSqlite::in_memory().await.unwrap();
        let mut transaction = sqlite.transaction().await.unwrap();

        let mut branch_name = RoswaalOwnedGitBranchName::new("test");
        let mut locations = vec![RoswaalLocation::new_without_validation("New York", 45.0, 45.0)];
        transaction.save_locations(&locations, &branch_name).await.unwrap();
        transaction.merge_unmerged_locations(&branch_name).await.unwrap();

        branch_name = RoswaalOwnedGitBranchName::new("test-2");
        locations = vec![RoswaalLocation::new_without_validation("New York", 82.2987299, -6.90872987)];
        transaction.save_locations(&locations, &branch_name).await.unwrap();
        transaction.merge_unmerged_locations(&branch_name).await.unwrap();

        let saved_locations = transaction.locations_in_alphabetical_order(LoadLocationsFilter::All)
            .await.unwrap();
        let expected_locations = vec![
            RoswaalStoredLocation { location: locations[0].clone(), unmerged_branch_name: None }
        ];
        assert_eq!(saved_locations, expected_locations)
    }

    #[tokio::test]
    async fn loads_only_merged_locations_with_merge_only_filter() {
        let sqlite = RoswaalSqlite::in_memory().await.unwrap();
        let mut transaction = sqlite.transaction().await.unwrap();

        let mut branch_name = RoswaalOwnedGitBranchName::new("test");
        let new_york = RoswaalLocation::new_without_validation("New York", 45.0, 45.0);
        let mut locations = vec![new_york.clone()];
        transaction.save_locations(&locations, &branch_name).await.unwrap();
        transaction.merge_unmerged_locations(&branch_name).await.unwrap();

        branch_name = RoswaalOwnedGitBranchName::new("test-2");
        locations = vec![RoswaalLocation::new_without_validation("Antarctica", 82.2987299, -6.90872987)];
        transaction.save_locations(&locations, &branch_name).await.unwrap();

        let saved_locations = transaction.locations_in_alphabetical_order(LoadLocationsFilter::MergedOnly)
            .await.unwrap();
        let expected_locations = vec![
            RoswaalStoredLocation { location: new_york, unmerged_branch_name: None }
        ];
        assert_eq!(saved_locations, expected_locations)
    }
}
