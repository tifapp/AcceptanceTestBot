use anyhow::Result;
use std::{path::Path, sync::Arc};
use git2::{build::CheckoutBuilder, Cred, FetchOptions, IndexAddOption, PushOptions, RemoteCallbacks, Repository, ResetType};
use tokio::sync::{Mutex, MutexGuard};

use super::{branch_name::RoswaalOwnedGitBranchName, metadata::RoswaalGitRepositoryMetadata};

/// A wrapper for a git repository that serializes access to an underlying git client.
pub struct RoswaalGitRepository<Client> {
    mutex: Arc<Mutex<Client>>
}

impl <Client> RoswaalGitRepository<Client>
    where Client: RoswaalGitRepositoryClient {
    /// Attempts to open a repository with the specified metadata.
    pub async fn open(metadata: &RoswaalGitRepositoryMetadata) -> Result<Self> {
        let client = Client::try_new(metadata).await?;
        Ok(Self { mutex: Arc::new(Mutex::new(client)) })
    }
}

pub type RoswaalGitRepositoryTransaction<'a, Client> = MutexGuard<'a, Client>;

impl <Client> RoswaalGitRepository<Client>
    where Client: RoswaalGitRepositoryClient {
    /// Starts a transaction to this repository.
    pub async fn transaction(&self) -> RoswaalGitRepositoryTransaction<Client> {
        self.mutex.lock().await
    }
}

/// A git client trait.
pub trait RoswaalGitRepositoryClient: Sized {
    /// Attempts to create this client from metadata.
    async fn try_new(metadata: &RoswaalGitRepositoryMetadata) -> Result<Self>;

    /// Performs the equivalent of a `git reset --hard HEAD`.
    async fn hard_reset_to_head(&self) -> Result<()>;

    /// Performs the equivalent of a `git switch <branch>`.
    async fn switch_branch(&self, name: &str) -> Result<()>;

    /// Performs the equivalent of a `git pull origin <branch>`.
    async fn pull_branch(&self, name: &str) -> Result<()>;

    /// Performs the equivalent of a `git commit -am <message>`.
    async fn commit_all(&self, message: &str) -> Result<()>;

    /// Performs the equivalent of a `git checkout -b <branch>`.
    async fn checkout_new_branch(&self, name: &RoswaalOwnedGitBranchName) -> Result<()>;

    /// Peforms the equivalent of a `git push origin <branch>`.
    async fn push_changes(&self, branch_name: &RoswaalOwnedGitBranchName) -> Result<()>;
}

/// A `RoswaalGitRepositoryClient` implementation using lib2git and the git2 crate.
pub struct LibGit2RepositoryClient {
    repo: Repository,
    metadata: RoswaalGitRepositoryMetadata
}

impl RoswaalGitRepositoryClient for LibGit2RepositoryClient {
    async fn try_new(metadata: &RoswaalGitRepositoryMetadata) -> Result<Self> {
        let repo = Repository::open(metadata.relative_path("."))?;
        Ok(Self { repo, metadata: metadata.clone() })
    }

    async fn hard_reset_to_head(&self) -> Result<()> {
        let obj = self.repo.revparse_single("HEAD")?;
        self.repo.reset(&obj, ResetType::Hard, None)?;
        Ok(())
    }

    async fn clean_all_untracked() -> Result<()> {
        Ok(())
    }

    async fn switch_branch(&self, name: &str) -> Result<()> {
        self.repo.set_head(&format!("refs/heads/{}", name))?;
        let mut checkout_builder = CheckoutBuilder::new();
        self.repo.checkout_head(Some(&mut checkout_builder.force()))?;
        Ok(())
    }

    async fn pull_branch(&self, name: &str) -> Result<()> {
        let mut remote = self.repo.find_remote("origin")?;
        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(self.remote_callbacks());
        remote.fetch(&[name], Some(&mut fetch_options), None)?;
        let fetch_head = self.repo.find_reference("FETCH_HEAD")?;
        let fetch_commit = self.repo.reference_to_annotated_commit(&fetch_head)?;
        self.repo.merge(&[&fetch_commit], None, None)?;
        Ok(())
    }

    async fn commit_all(&self, message: &str) -> Result<()> {
        let mut index = self.repo.index()?;
        index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;
        index.write()?;
        let oid = index.write_tree()?;
        let signature = self.repo.signature()?;
        let parent_commit = self.repo.head()?.peel_to_commit()?;
        let tree = self.repo.find_tree(oid)?;
        self.repo.commit(Some("HEAD"), &signature, &signature, message, &tree, &[&parent_commit])?;
        Ok(())
    }

    async fn checkout_new_branch(&self, name: &RoswaalOwnedGitBranchName) -> Result<()> {
        let commit = self.repo.head()?.peel_to_commit()?;
        _ = self.repo.branch(&name.to_string(), &commit, false)?;
        let (object, reference) = self.repo.revparse_ext(&name.to_string())?;
        self.repo.checkout_tree(&object, None)?;
        if let Some(name) = reference.and_then(|r| r.name().map(|s| s.to_string())) {
            self.repo.set_head(&name)?;
        }
        Ok(())
    }

    async fn push_changes(&self, branch_name: &RoswaalOwnedGitBranchName) -> Result<()> {
        let mut remote = self.repo.find_remote("origin")?;
        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(self.remote_callbacks());
        remote.push(&[format!("refs/heads/{}", branch_name.to_string())], Some(&mut push_options))?;
        Ok(())
    }
}

impl LibGit2RepositoryClient {
    fn remote_callbacks(&self) -> RemoteCallbacks {
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_, user, _| {
            let private_key_path = self.metadata.ssh_private_key_path();
            Cred::ssh_key(user.unwrap(), None, Path::new(&private_key_path), None)
        });
        callbacks
    }
}

/// A `RoswaalGitRepositoryClient` implementation suitable for test-stubbing.
#[cfg(test)]
pub struct NoopGitRepositoryClient;

#[cfg(test)]
impl RoswaalGitRepositoryClient for NoopGitRepositoryClient {
    async fn try_new(_: &RoswaalGitRepositoryMetadata) -> Result<Self> {
        Ok(Self)
    }

    async fn hard_reset_to_head(&self) -> Result<()> {
        Ok(())
    }

    async fn clean_all_untracked() -> Result<()> {
        Ok(())
    }

    async fn switch_branch(&self, _: &str) -> Result<()> {
        Ok(())
    }

    async fn pull_branch(&self, _: &str) -> Result<()> {
        Ok(())
    }

    async fn commit_all(&self, _: &str) -> Result<()> {
        Ok(())
    }

    async fn checkout_new_branch(&self, _: &RoswaalOwnedGitBranchName) -> Result<()> {
        Ok(())
    }

    async fn push_changes(&self, _: &RoswaalOwnedGitBranchName) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::{fs::File, io::{AsyncReadExt, AsyncWriteExt}};

    use crate::{git::{branch_name::RoswaalOwnedGitBranchName, metadata::RoswaalGitRepositoryMetadata, repo::RoswaalGitRepository}, utils::test_support::with_test_repo_access};

    #[tokio::test]
    async fn test_add_commit_push_pull() {
        with_test_repo_access(async {
            let metadata = RoswaalGitRepositoryMetadata::for_testing();
            let repo = RoswaalGitRepository::<LibGit2RepositoryClient>::open(&metadata).await?;
            let transaction = repo.transaction().await;
            let branch_name = RoswaalOwnedGitBranchName::new("test");
            transaction.checkout_new_branch(&branch_name).await?;

            let expected_file_contents = "In this world, all life will walk towards the future, hand in hand.";
            let mut file = File::create(metadata.relative_path("test.txt")).await?;
            file.write(expected_file_contents.as_bytes()).await?;
            file.flush().await?;
            drop(file);

            transaction.commit_all("I like this!").await?;
            transaction.push_changes(&branch_name).await?;
            transaction.switch_branch(metadata.base_branch_name()).await?;

            assert!(File::open(metadata.relative_path("test.txt")).await.is_err());

            transaction.pull_branch(&branch_name.to_string()).await?;

            file = File::open(metadata.relative_path("test.txt")).await?;
            let mut file_contents = String::new();
            file.read_to_string(&mut file_contents).await?;

            assert_eq!(file_contents, expected_file_contents);
            Ok(())
        }).await.unwrap();
    }

    #[tokio::test]
    async fn test_reset_hard_to_head() {
        with_test_repo_access(async {
            let metadata = RoswaalGitRepositoryMetadata::for_testing();
            let repo = RoswaalGitRepository::<LibGit2RepositoryClient>::open(&metadata).await?;
            let transaction = repo.transaction().await;

            let mut file = File::create(metadata.relative_path("roswaal/Locations.ts")).await?;
            file.write("console.log(\"Hello world\")".as_bytes()).await?;
            file.flush().await?;
            drop(file);

            transaction.hard_reset_to_head().await?;

            file = File::open(metadata.relative_path("roswaal/Locations.ts")).await?;
            let mut contents = String::new();
            file.read_to_string(&mut contents).await?;
            assert!(contents.is_empty());
            Ok(())
        }).await.unwrap();
    }
}
