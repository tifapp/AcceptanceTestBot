use crate::{
    location::storage::{LoadLocationsFilter, RoswaalStoredLocation},
    utils::sqlite::RoswaalSqlite,
    with_transaction,
};
use anyhow::Result;

#[derive(Debug, PartialEq)]
pub enum LoadAllLocationsStatus {
    Success(Vec<RoswaalStoredLocation>),
    NoLocations,
}

impl LoadAllLocationsStatus {
    pub async fn from_stored_locations(sqlite: &RoswaalSqlite) -> Result<Self> {
        let mut transaction = sqlite.transaction().await?;
        with_transaction!(transaction, async {
            transaction
                .locations_in_alphabetical_order(LoadLocationsFilter::All)
                .await
                .map(|locations| {
                    if locations.is_empty() {
                        Self::NoLocations
                    } else {
                        Self::Success(locations)
                    }
                })
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        git::{
            branch_name,
            repo::RoswaalGitRepository,
            test_support::{with_clean_test_repo_access, TestGithubPullRequestOpen},
        },
        is_case,
        location::location::RoswaalLocation,
        operations::{
            add_locations::AddLocationsStatus, close_branch::CloseBranchStatus,
            load_all_locations::LoadAllLocationsStatus,
        },
        utils::sqlite::RoswaalSqlite,
    };

    #[tokio::test]
    async fn test_returns_no_locations_when_no_saved_locations() {
        let sqlite = RoswaalSqlite::in_memory().await.unwrap();
        let status = LoadAllLocationsStatus::from_stored_locations(&sqlite)
            .await
            .unwrap();
        assert_eq!(status, LoadAllLocationsStatus::NoLocations)
    }

    #[tokio::test]
    async fn test_returns_locations_from_add_operation() {
        with_clean_test_repo_access(async {
            let sqlite = RoswaalSqlite::in_memory().await.unwrap();
            let expected_locations = vec![
                RoswaalLocation::new_without_validation("Test 1", 50.0, 50.0),
                RoswaalLocation::new_without_validation("Test 2", -5.0, 5.0),
            ];
            let locations_str = "
Test 1, 50.0, 50.0
Invalid
Test 2, -15.0, 15.0
Test 2, -5.0, 5.0
                ";
            _ = AddLocationsStatus::from_adding_locations(
                &locations_str,
                &RoswaalGitRepository::noop().await?,
                &sqlite,
                &TestGithubPullRequestOpen::new(false),
            )
            .await?;
            let status = LoadAllLocationsStatus::from_stored_locations(&sqlite)
                .await
                .unwrap();
            assert!(is_case!(status, LoadAllLocationsStatus::Success));
            assert_eq!(status.locations(), expected_locations);
            Ok(())
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_returns_no_locations_when_opening_add_locations_pr_fails() {
        with_clean_test_repo_access(async {
            let sqlite = RoswaalSqlite::in_memory().await.unwrap();
            let locations_str = "
Test 1, 50.0, 50.0
Test 2, -5.0, 5.0
                ";
            let pr_open = TestGithubPullRequestOpen::new(true);
            _ = AddLocationsStatus::from_adding_locations(
                &locations_str,
                &RoswaalGitRepository::noop().await?,
                &sqlite,
                &pr_open,
            )
            .await?;
            let status = LoadAllLocationsStatus::from_stored_locations(&sqlite)
                .await
                .unwrap();
            assert_eq!(status, LoadAllLocationsStatus::NoLocations);
            Ok(())
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_returns_no_locations_when_add_locations_merge_conflict_occurs() {
        with_clean_test_repo_access(async {
            let sqlite = RoswaalSqlite::in_memory().await.unwrap();
            let locations_str = "
Test 1, 50.0, 50.0
Test 2, -5.0, 5.0
                ";
            let repo = RoswaalGitRepository::noop().await?;
            let mut transaction = repo.transaction().await;
            transaction.ensure_merge_conflict();
            drop(transaction);
            let pr_open = TestGithubPullRequestOpen::new(false);
            _ = AddLocationsStatus::from_adding_locations(&locations_str, &repo, &sqlite, &pr_open)
                .await?;
            let status = LoadAllLocationsStatus::from_stored_locations(&sqlite)
                .await
                .unwrap();
            assert_eq!(status, LoadAllLocationsStatus::NoLocations);
            Ok(())
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn reports_no_loactions_when_no_merged_locations_and_all_unmerged_locations_are_closed() {
        with_clean_test_repo_access(async {
            let sqlite = RoswaalSqlite::in_memory().await.unwrap();
            let locations_str = "
Test 1, 50.0, 50.0
Test 2, -5.0, 5.0
                ";
            let repo = RoswaalGitRepository::noop().await?;
            let pr_open = TestGithubPullRequestOpen::new(false);
            AddLocationsStatus::from_adding_locations(&locations_str, &repo, &sqlite, &pr_open)
                .await?;
            let branch_name = pr_open.most_recent_head_branch_name().await.unwrap();
            CloseBranchStatus::from_closing_branch(&branch_name, &sqlite).await?;
            let status = LoadAllLocationsStatus::from_stored_locations(&sqlite)
                .await
                .unwrap();
            assert_eq!(status, LoadAllLocationsStatus::NoLocations);
            Ok(())
        })
        .await
        .unwrap();
    }

    impl LoadAllLocationsStatus {
        fn locations(&self) -> Vec<RoswaalLocation> {
            match self {
                Self::Success(locations) => {
                    locations.iter().map(|l| l.location().clone()).collect()
                }
                _ => panic!("Must be a success status"),
            }
        }
    }
}
