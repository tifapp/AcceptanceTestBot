use anyhow::Result;
use sqlx::{query, query_as, FromRow, Sqlite};

use crate::{git::branch_name::{self, RoswaalOwnedGitBranchName}, utils::sqlite::RoswaalSqliteTransaction};

use super::location::RoswaalLocation;

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

const SAVE_STATEMENT: &str = "
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

const UPDATE_MERGE_UNMERGED_STATEMENT: &str = "
DELETE FROM Locations WHERE name = ? AND unmerged_branch_name IS NULL;
UPDATE Locations SET unmerged_branch_name = NULL WHERE unmerged_branch_name = ? AND name = ?;
";

impl <'a> RoswaalSqliteTransaction <'a> {
    pub async fn merge_unmerged_locations(&mut self, branch_name: &RoswaalOwnedGitBranchName) -> Result<()> {
        let sqlite_location_names = query_as::<Sqlite, SqliteLocationName>(
            "SELECT name FROM Locations WHERE unmerged_branch_name = ?;"
        )
        .bind(branch_name.to_string())
        .fetch_all(self.connection())
        .await?;
        let update_statements = sqlite_location_names.iter().map(|_| UPDATE_MERGE_UNMERGED_STATEMENT)
            .collect::<Vec<&str>>()
            .join("\n");
        let mut update_query = query::<Sqlite>(&update_statements);
        for sqlite_name in sqlite_location_names.iter() {
            update_query = update_query.bind(sqlite_name.name.clone())
                .bind(branch_name.to_string())
                .bind(sqlite_name.name.clone());
        }
        update_query.execute(self.connection()).await?;
        Ok(())
    }

    pub async fn save_locations(
        &mut self,
        locations: &Vec<RoswaalLocation>,
        branch_name: &RoswaalOwnedGitBranchName
    ) -> Result<()> {
        let statements = locations.iter()
            .map(|_| SAVE_STATEMENT)
            .collect::<Vec<&str>>()
            .join("\n");
        let mut bulk_insert_query = query::<Sqlite>(&statements);
        for location in locations.iter() {
            bulk_insert_query = bulk_insert_query.bind(location.coordinate().latitude())
                .bind(location.coordinate().longitude())
                .bind(&location.name().raw_value)
                .bind(branch_name.to_string())
        }
        bulk_insert_query.execute(self.connection()).await?;
        Ok(())
    }

    pub async fn locations_in_alphabetical_order(&mut self) -> Result<Vec<RoswaalStoredLocation>> {
        let locations = query_as::<Sqlite, SqliteLocation>(
            "SELECT * FROM Locations ORDER BY name, latitude"
        )
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
        let saved_locations = transaction.locations_in_alphabetical_order().await.unwrap();
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
        let saved_locations = transaction.locations_in_alphabetical_order().await.unwrap();
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
        let saved_locations = transaction.locations_in_alphabetical_order().await.unwrap();
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
        let saved_locations = transaction.locations_in_alphabetical_order().await.unwrap();
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

        let saved_locations = transaction.locations_in_alphabetical_order().await.unwrap();
        let expected_locations = vec![
            RoswaalStoredLocation { location: locations[0].clone(), unmerged_branch_name: None }
        ];
        assert_eq!(saved_locations, expected_locations)
    }
}
