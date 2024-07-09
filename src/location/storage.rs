use anyhow::Result;
use sqlx::{query, query_as, FromRow, Sqlite};

use crate::{git::branch_name::RoswaalOwnedGitBranchName, utils::sqlite::RoswaalSqliteTransaction};

use super::location::RoswaalLocation;

#[derive(Debug, PartialEq)]
pub struct RoswaalStoredLocation {
    location: RoswaalLocation,
    branch_name: Option<RoswaalOwnedGitBranchName>
}

impl RoswaalStoredLocation {
    pub fn location(&self) -> &RoswaalLocation {
        &self.location
    }

    pub fn branch_name(&self) -> Option<&RoswaalOwnedGitBranchName> {
        self.branch_name.as_ref()
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

impl <'a> RoswaalSqliteTransaction <'a> {
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
            branch_name: l.unmerged_branch_name.clone()
        })
        .collect();
        Ok(locations)
    }
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
            RoswaalStoredLocation { location: locations[0].clone(), branch_name: Some(branch_name.clone()) },
            RoswaalStoredLocation { location: locations[1].clone(), branch_name: Some(branch_name.clone()) }
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
            RoswaalStoredLocation { location: locations[0].clone(), branch_name: Some(branch_name) }
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
                branch_name: Some(branch_name.clone())
            },
            RoswaalStoredLocation {
                location: RoswaalLocation::new_without_validation("Antarctica", 50.0, 50.0),
                branch_name: Some(branch_name2.clone())
            },
            RoswaalStoredLocation {
                location: RoswaalLocation::new_without_validation("New York", 45.0, 45.0),
                branch_name: Some(branch_name.clone())
            },
            RoswaalStoredLocation {
                location: RoswaalLocation::new_without_validation("Oakland", 45.0, 45.0),
                branch_name: Some(branch_name2.clone())
            }
        ];
        assert_eq!(saved_locations, expected_locations)
    }
}
