use anyhow::Result;
use tokio::{fs::File, io::AsyncWriteExt};

use crate::{generation::interface::RoswaalTypescriptGenerate, git::{branch_name::RoswaalOwnedGitBranchName, edit::EditGitRepositoryStatus, pull_request::GithubPullRequestOpen, repo::{RoswaalGitRepository, RoswaalGitRepositoryClient}}, location::{location::{RoswaalLocation, RoswaalStringLocations}, storage::RoswaalStoredLocation}, utils::sqlite::RoswaalSqlite, with_transaction};

#[derive(Debug, PartialEq)]
pub enum AddLocationsStatus {
    Success { locations: RoswaalStringLocations, did_delete_branch: bool },
    NoLocationsAdded,
    FailedToOpenPullRequest,
    MergeConflict
}

impl AddLocationsStatus {
    pub async fn from_adding_locations<GitClient: RoswaalGitRepositoryClient>(
        locations_str: &str,
        git_repository: &RoswaalGitRepository<GitClient>,
        sqlite: &RoswaalSqlite,
        pr_open: &impl GithubPullRequestOpen
    ) -> Result<Self> {
        if locations_str.is_empty() {
            return Ok(Self::NoLocationsAdded)
        }
        let string_locations = RoswaalStringLocations::from_roswaal_locations_str(locations_str);
        let branch_name = RoswaalOwnedGitBranchName::for_adding_locations();
        let mut transaction = sqlite.transaction().await?;
        let (stored_locations, git_transaction) = with_transaction!(transaction, async {
            let locations = transaction.locations_in_alphabetical_order().await?;
            Ok((locations, git_repository.transaction().await))
        })?;

        let metadata = git_transaction.metadata().clone();
        let edit_status = EditGitRepositoryStatus::from_editing_new_branch(
            &branch_name,
            git_transaction,
            pr_open,
            async {
                Self::generate_locations_code(
                    &string_locations,
                    &stored_locations,
                    metadata.locations_path()
                ).await?;
                Ok(metadata.add_locations_pull_request(&string_locations, &branch_name))
            }
        ).await?;

        match edit_status {
            EditGitRepositoryStatus::Success { did_delete_branch } => {
                transaction = sqlite.transaction().await?;
                with_transaction!(transaction, async {
                    transaction.save_locations(&string_locations.locations(), &branch_name).await?;
                    Ok(Self::Success { locations: string_locations, did_delete_branch })
                })
            },
            EditGitRepositoryStatus::FailedToOpenPullRequest => {
                Ok(Self::FailedToOpenPullRequest)
            },
            EditGitRepositoryStatus::MergeConflict => {
                Ok(Self::MergeConflict)
            }
        }
    }

    async fn generate_locations_code(
        string_locations: &RoswaalStringLocations,
        stored_locations: &Vec<RoswaalStoredLocation>,
        path: &str
    ) -> Result<()> {
        let locations_code = stored_locations.iter()
            .map(|l| l.location())
            .chain(string_locations.locations().iter())
            .collect::<Vec<&RoswaalLocation>>()
            .typescript();
        let mut file = File::create(path).await?;
        file.write(locations_code.as_bytes()).await?;
        file.flush().await?;
        drop(file);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{git::{metadata::{self, RoswaalGitRepositoryMetadata}, repo::RoswaalGitRepository, test_support::{read_string, with_clean_test_repo_access, TestGithubPullRequestOpen}}, is_case, location::location::RoswaalStringLocations, operations::add_locations::AddLocationsStatus, utils::sqlite::RoswaalSqlite};

    #[tokio::test]
    async fn test_success_when_adding_locations_smoothly() {
        with_clean_test_repo_access(async {
            let str = "Test, 50.0, 50.0";
            let sqlite = RoswaalSqlite::in_memory().await?;
            let result = AddLocationsStatus::from_adding_locations(
                str,
                &RoswaalGitRepository::noop().await?,
                &sqlite,
                &TestGithubPullRequestOpen::new(false)
            ).await?;
            let str_locations = RoswaalStringLocations::from_roswaal_locations_str(str);
            assert_eq!(result, AddLocationsStatus::Success { locations: str_locations, did_delete_branch: true });
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
                &RoswaalGitRepository::noop().await.unwrap(),
                &sqlite,
                &TestGithubPullRequestOpen::new(false)
            ).await?;
            let str_locations = RoswaalStringLocations::from_roswaal_locations_str(str);
            assert_eq!(result, AddLocationsStatus::Success { locations: str_locations, did_delete_branch: true });
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
                &RoswaalGitRepository::noop().await.unwrap(),
                &sqlite,
                &TestGithubPullRequestOpen::new(false)
            ).await;
            assert_eq!(result.ok(), Some(AddLocationsStatus::NoLocationsAdded));
            Ok(())
        })
        .await.unwrap();
    }

    #[tokio::test]
    async fn test_creates_code_file_with_existing_locations() {
        with_clean_test_repo_access(async {
            let metadata = RoswaalGitRepositoryMetadata::for_testing();
            let sqlite = RoswaalSqlite::in_memory().await.unwrap();
            _ = AddLocationsStatus::from_adding_locations(
                "Test, 50.0, 50.0",
                &RoswaalGitRepository::noop().await.unwrap(),
                &sqlite,
                &TestGithubPullRequestOpen::new(false)
            ).await;
            _ = AddLocationsStatus::from_adding_locations(
                "Test 2, 45.0, 45.0",
                &RoswaalGitRepository::noop().await.unwrap(),
                &sqlite,
                &TestGithubPullRequestOpen::new(false)
            ).await;
            let content = read_string(metadata.locations_path()).await?;
            let expected_content = "\
// Generated by Roswaal, do not touch.

export namespace TestLocations {
  export const Test = {
    latitude: 50.0000000000000000,
    longitude: 50.0000000000000000
  }
  export const Test2 = {
    latitude: 45.0000000000000000,
    longitude: 45.0000000000000000
  }
}
";
            assert_eq!(&content, expected_content);
            Ok(())
        })
        .await.unwrap();
    }

    #[tokio::test]
    async fn test_opens_pr_with_new_locations() {
        with_clean_test_repo_access(async {
            let pr_open = TestGithubPullRequestOpen::new(false);
            let sqlite = RoswaalSqlite::in_memory().await.unwrap();
            _ = AddLocationsStatus::from_adding_locations(
                "Test, 50.0, 50.0",
                &RoswaalGitRepository::noop().await.unwrap(),
                &sqlite,
                &pr_open
            ).await?;
            let pr = pr_open.most_recent_pr().await.unwrap();
            assert!(pr.title().contains("Add Locations (Test)"));
            Ok(())
        })
        .await.unwrap()
    }

    #[tokio::test]
    async fn test_returns_pr_failed_status_when_pr_open_fails() {
        with_clean_test_repo_access(async {
            let pr_open = TestGithubPullRequestOpen::new(true);
            let sqlite = RoswaalSqlite::in_memory().await.unwrap();
            let result = AddLocationsStatus::from_adding_locations(
                "Test, 50.0, 50.0",
                &RoswaalGitRepository::noop().await.unwrap(),
                &sqlite,
                &pr_open
            ).await?;
            assert_eq!(result, AddLocationsStatus::FailedToOpenPullRequest);
            Ok(())
        })
        .await.unwrap()
    }

    #[tokio::test]
    async fn test_returns_merge_conflict_status_when_merge_conflict_occurs() {
        with_clean_test_repo_access(async {
            let pr_open = TestGithubPullRequestOpen::new(true);
            let sqlite = RoswaalSqlite::in_memory().await.unwrap();
            let repo = RoswaalGitRepository::noop().await?;
            let mut transaction = repo.transaction().await;
            transaction.ensure_merge_conflict();
            drop(transaction);
            let result = AddLocationsStatus::from_adding_locations(
                "Test, 50.0, 50.0",
                &repo,
                &sqlite,
                &pr_open
            ).await?;
            assert_eq!(result, AddLocationsStatus::MergeConflict);
            Ok(())
        })
        .await.unwrap()
    }
}
