use anyhow::Result;

use crate::{git::{branch_name::{self, RoswaalOwnedGitBranchName}, repo::{RoswaalGitRepository, RoswaalGitRepositoryClient}}, location::location::RoswaalStringLocations, utils::sqlite::RoswaalSqlite, with_transaction};

#[derive(Debug, PartialEq)]
pub enum AddLocationsStatus {
    Success(RoswaalStringLocations),
    NoLocationsAdded
}

impl AddLocationsStatus {
    pub async fn from_adding_locations(
        locations_str: &str,
        git_repository: RoswaalGitRepository<impl RoswaalGitRepositoryClient>,
        sqlite: &RoswaalSqlite
    ) -> Result<Self> {
        if locations_str.is_empty() {
            return Ok(Self::NoLocationsAdded)
        }
        let string_locations = RoswaalStringLocations::from_roswaal_locations_str(locations_str);
        let mut transaction = sqlite.transaction().await?;
        with_transaction!(transaction, async {
            let branch_name = RoswaalOwnedGitBranchName::for_adding_locations();
            transaction.save_locations(&string_locations.locations(), &branch_name).await?;
            Ok(Self::Success(string_locations))
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{git::{repo::RoswaalGitRepository, test_support::with_clean_test_repo_access}, location::location::RoswaalStringLocations, operations::add_locations::AddLocationsStatus, utils::sqlite::RoswaalSqlite};

    #[tokio::test]
    async fn test_success_when_adding_locations_smoothly() {
        with_clean_test_repo_access(async {
            let str = "Test, 50.0, 50.0";
            let sqlite = RoswaalSqlite::in_memory().await?;
            let result = AddLocationsStatus::from_adding_locations(
                str,
                RoswaalGitRepository::noop().await?,
                &sqlite
            ).await;
            let str_locations = RoswaalStringLocations::from_roswaal_locations_str(str);
            assert_eq!(result.ok(), Some(AddLocationsStatus::Success(str_locations)));
            Ok(())
        })
        .await.unwrap()
    }

    #[tokio::test]
    async fn test_success_mixes_proper_and_invalid_locations() {
        with_clean_test_repo_access(async {
            let str = "Test, 50.0, 50.0\n29879";
            let sqlite = RoswaalSqlite::in_memory().await.unwrap();
            let result = AddLocationsStatus::from_adding_locations(
                str,
                RoswaalGitRepository::noop().await.unwrap(),
                &sqlite
            ).await;
            let str_locations = RoswaalStringLocations::from_roswaal_locations_str(str);
            assert_eq!(result.ok(), Some(AddLocationsStatus::Success(str_locations)));
            Ok(())
        })
        .await.unwrap();
    }

    #[tokio::test]
    async fn test_no_locations_added_when_empty_vector() {
        with_clean_test_repo_access(async {
            let sqlite = RoswaalSqlite::in_memory().await.unwrap();
            let result = AddLocationsStatus::from_adding_locations(
                "",
                RoswaalGitRepository::noop().await.unwrap(),
                &sqlite
            ).await;
            assert_eq!(result.ok(), Some(AddLocationsStatus::NoLocationsAdded));
            Ok(())
        })
        .await.unwrap();
    }
}
