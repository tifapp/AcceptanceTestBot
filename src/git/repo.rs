use anyhow::Result;
use git2::{
    build::CheckoutBuilder, AnnotatedCommit, BranchType, Cred, FetchOptions, IndexAddOption,
    PushOptions, RemoteCallbacks, Repository, ResetType,
};
use std::{
    path::{Path, PathBuf},
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc,
    },
    thread,
};
use tokio::{
    fs::remove_file,
    spawn,
    sync::{oneshot, Mutex, MutexGuard},
    task::spawn_blocking,
};

use crate::utils::fs::remove_dir_all_empty;

use super::{branch_name::RoswaalOwnedGitBranchName, metadata::RoswaalGitRepositoryMetadata};

/// A wrapper for a git repository that serializes access to an underlying git client.
pub struct RoswaalGitRepository<Client> {
    mutex: Arc<Mutex<Client>>,
}

impl<Client> RoswaalGitRepository<Client>
where
    Client: RoswaalGitRepositoryClient,
{
    /// Attempts to open a repository with the specified metadata.
    pub async fn open(metadata: &RoswaalGitRepositoryMetadata) -> Result<Self> {
        let client = Client::try_new(metadata).await?;
        Ok(Self {
            mutex: Arc::new(Mutex::new(client)),
        })
    }
}

pub type RoswaalGitRepositoryTransaction<'a, Client> = MutexGuard<'a, Client>;

impl<Client> RoswaalGitRepository<Client>
where
    Client: RoswaalGitRepositoryClient,
{
    /// Starts a transaction to this repository.
    pub async fn transaction(&self) -> RoswaalGitRepositoryTransaction<Client> {
        self.mutex.lock().await
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum PullBranchStatus {
    Success,
    MergeConflict,
}

type MergeBranchStatus = PullBranchStatus;

/// A git client trait.
pub trait RoswaalGitRepositoryClient: Sized {
    /// Attempts to create this client from metadata.
    async fn try_new(metadata: &RoswaalGitRepositoryMetadata) -> Result<Self>;

    /// Returns the metadata associated with this client.
    fn metadata(&self) -> &RoswaalGitRepositoryMetadata;

    /// Performs the equivalent of a `git reset --hard HEAD`.
    async fn hard_reset_to_head(&self) -> Result<()>;

    /// Performs the equivalent of a `git clean -fd`.
    async fn clean_all_untracked(&self) -> Result<()>;

    /// Performs the equivalent of a `git switch <branch>`.
    async fn switch_branch(&self, name: &str) -> Result<()>;

    /// Performs the equivalent of a `git pull origin <branch>`.
    async fn pull_branch(&self, name: &str) -> Result<PullBranchStatus>;

    /// Performs the equivalent of a `git commit -am <message>`.
    async fn commit_all(&self, message: &str) -> Result<()>;

    /// Performs the equivalent of a `git checkout -b <branch>`.
    async fn checkout_new_branch(&self, name: &RoswaalOwnedGitBranchName) -> Result<()>;

    /// Peforms the equivalent of a `git push origin <branch>`.
    async fn push_changes(&self, branch_name: &RoswaalOwnedGitBranchName) -> Result<()>;

    /// Performs the equivalent of a `git branch -d <branch>`.
    ///
    /// Returns true if the deletion was successful.
    async fn delete_local_branch(&self, branch_name: &RoswaalOwnedGitBranchName) -> Result<bool>;
}

/// A `RoswaalGitRepositoryClient` implementation using lib2git and the git2 crate.
pub struct LibGit2RepositoryClient {
    sender: Sender<LibGit2ThreadRequest>,
    metadata: RoswaalGitRepositoryMetadata,
}

enum LibGit2ThreadRequest {
    HardResetToHead {
        sender: oneshot::Sender<Result<()>>,
    },
    SwitchBranch {
        name: String,
        sender: oneshot::Sender<Result<()>>,
    },
    PullBranch {
        name: String,
        sender: oneshot::Sender<Result<PullBranchStatus>>,
    },
    CommitAll {
        message: String,
        sender: oneshot::Sender<Result<()>>,
    },
    CheckoutNewBranch {
        name: RoswaalOwnedGitBranchName,
        sender: oneshot::Sender<Result<()>>,
    },
    PushChanges {
        name: RoswaalOwnedGitBranchName,
        sender: oneshot::Sender<Result<()>>,
    },
    Statuses {
        sender: oneshot::Sender<Result<Vec<LibGit2StatusEntry>>>,
    },
    DeleteLocalBranch {
        name: RoswaalOwnedGitBranchName,
        sender: oneshot::Sender<Result<bool>>,
    },
}

struct LibGit2StatusEntry {
    path: PathBuf,
}

impl RoswaalGitRepositoryClient for LibGit2RepositoryClient {
    async fn try_new(metadata: &RoswaalGitRepositoryMetadata) -> Result<Self> {
        let m1 = metadata.clone();
        let (tx, rx) = channel::<LibGit2ThreadRequest>();
        let repo = spawn_blocking(move || Repository::open(m1.relative_path("."))).await??;
        Self::thread(repo, metadata, rx);
        Ok(Self {
            sender: tx,
            metadata: metadata.clone(),
        })
    }

    fn metadata(&self) -> &RoswaalGitRepositoryMetadata {
        &self.metadata
    }

    async fn hard_reset_to_head(&self) -> Result<()> {
        let (sender, receiver) = oneshot::channel::<Result<()>>();
        self.sender
            .send(LibGit2ThreadRequest::HardResetToHead { sender })?;
        receiver.await?
    }

    async fn switch_branch(&self, name: &str) -> Result<()> {
        let (sender, receiver) = oneshot::channel::<Result<()>>();
        self.sender.send(LibGit2ThreadRequest::SwitchBranch {
            name: name.to_string(),
            sender,
        })?;
        receiver.await?
    }

    async fn pull_branch(&self, name: &str) -> Result<PullBranchStatus> {
        let (sender, receiver) = oneshot::channel::<Result<PullBranchStatus>>();
        self.sender.send(LibGit2ThreadRequest::PullBranch {
            name: name.to_string(),
            sender,
        })?;
        receiver.await?
    }

    async fn commit_all(&self, message: &str) -> Result<()> {
        let (sender, receiver) = oneshot::channel::<Result<()>>();
        self.sender.send(LibGit2ThreadRequest::CommitAll {
            message: message.to_string(),
            sender,
        })?;
        receiver.await?
    }

    async fn checkout_new_branch(&self, name: &RoswaalOwnedGitBranchName) -> Result<()> {
        let (sender, receiver) = oneshot::channel::<Result<()>>();
        self.sender.send(LibGit2ThreadRequest::CheckoutNewBranch {
            name: name.clone(),
            sender,
        })?;
        receiver.await?
    }

    async fn push_changes(&self, branch_name: &RoswaalOwnedGitBranchName) -> Result<()> {
        let (sender, receiver) = oneshot::channel::<Result<()>>();
        self.sender.send(LibGit2ThreadRequest::PushChanges {
            name: branch_name.clone(),
            sender,
        })?;
        receiver.await?
    }

    async fn clean_all_untracked(&self) -> Result<()> {
        let (sender, receiver) = oneshot::channel::<Result<Vec<LibGit2StatusEntry>>>();
        self.sender
            .send(LibGit2ThreadRequest::Statuses { sender })?;
        let entries = receiver.await??;
        let futures = entries
            .iter()
            .map(|entry| spawn(remove_file(entry.path.clone())));
        for f in futures {
            f.await??;
        }
        remove_dir_all_empty(self.metadata.relative_path(".")).await?;
        Ok(())
    }

    async fn delete_local_branch(&self, branch_name: &RoswaalOwnedGitBranchName) -> Result<bool> {
        let (sender, receiver) = oneshot::channel::<Result<bool>>();
        self.sender.send(LibGit2ThreadRequest::DeleteLocalBranch {
            name: branch_name.clone(),
            sender,
        })?;
        receiver.await?
    }
}

// NB: libgit2 is not thread safe. In order to avoid blocking the cooperative thread pool, we'll
// need to run all operations on a dedicated background thread. spawn_blocking does not work since
// `Repository` does not implment Sync.
impl LibGit2RepositoryClient {
    fn thread(
        repo: Repository,
        metadata: &RoswaalGitRepositoryMetadata,
        receiver: Receiver<LibGit2ThreadRequest>,
    ) {
        let metadata = metadata.clone();
        thread::spawn(move || {
            for request in receiver {
                match request {
                    LibGit2ThreadRequest::HardResetToHead { sender } => {
                        _ = sender.send(Self::hard_reset_to_head(&repo));
                    }
                    LibGit2ThreadRequest::SwitchBranch { name, sender } => {
                        _ = sender.send(Self::switch_branch(&repo, &name));
                    }
                    LibGit2ThreadRequest::PullBranch { name, sender } => {
                        _ = sender.send(Self::pull_branch(
                            &repo,
                            &name,
                            metadata.remote_callbacks(),
                        ));
                    }
                    LibGit2ThreadRequest::CommitAll { message, sender } => {
                        _ = sender.send(Self::commit_all(&repo, &message));
                    }
                    LibGit2ThreadRequest::CheckoutNewBranch { name, sender } => {
                        _ = sender.send(Self::checkout_new_branch(&repo, &name));
                    }
                    LibGit2ThreadRequest::PushChanges { name, sender } => {
                        _ = sender.send(Self::push_changes(
                            &repo,
                            &name,
                            metadata.remote_callbacks(),
                        ));
                    }
                    LibGit2ThreadRequest::Statuses { sender } => {
                        _ = sender.send(Self::statuses(&repo));
                    }
                    LibGit2ThreadRequest::DeleteLocalBranch { name, sender } => {
                        _ = sender.send(Self::delete_local_branch(&repo, &name));
                    }
                }
            }
        });
    }

    fn hard_reset_to_head(repo: &Repository) -> Result<()> {
        let obj = repo.revparse_single("HEAD")?;
        repo.reset(&obj, ResetType::Hard, None)?;
        Ok(())
    }

    fn switch_branch(repo: &Repository, name: &str) -> Result<()> {
        repo.set_head(&format!("refs/heads/{}", name))?;
        let mut checkout_builder = CheckoutBuilder::new();
        repo.checkout_head(Some(&mut checkout_builder.force()))?;
        Ok(())
    }

    fn pull_branch(
        repo: &Repository,
        name: &str,
        callbacks: RemoteCallbacks,
    ) -> Result<PullBranchStatus> {
        Self::merge(repo, name, &Self::fetch(repo, name, callbacks)?)
    }

    fn fetch<'a>(
        repo: &'a Repository,
        branch_name: &str,
        callbacks: RemoteCallbacks,
    ) -> Result<AnnotatedCommit<'a>> {
        let mut remote = repo.find_remote("origin")?;
        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);
        remote.fetch(&[branch_name], Some(&mut fetch_options), None)?;
        let fetch_head = repo.find_reference("FETCH_HEAD")?;
        Ok(repo.reference_to_annotated_commit(&fetch_head)?)
    }

    fn merge(
        repo: &Repository,
        branch_name: &str,
        commit: &AnnotatedCommit,
    ) -> Result<MergeBranchStatus> {
        let (analysis, _) = repo.merge_analysis(&[commit])?;
        if analysis.is_fast_forward() {
            Self::merge_fast_forward(repo, branch_name, commit)
        } else if analysis.is_normal() {
            Self::merge_normal(repo, commit)
        } else {
            Ok(MergeBranchStatus::Success)
        }
    }

    fn merge_normal(repo: &Repository, commit: &AnnotatedCommit) -> Result<MergeBranchStatus> {
        repo.merge(&[&commit], None, None)?;
        Self::current_merge_status(repo)
    }

    fn merge_fast_forward(
        repo: &Repository,
        branch_name: &str,
        commit: &AnnotatedCommit,
    ) -> Result<MergeBranchStatus> {
        // NB: Adopted from https://github.com/rust-lang/git2-rs/blob/master/examples/pull.rs#L159
        let branch_reference_name = format!("refs/heads/{}", branch_name);
        match repo.find_reference(&branch_reference_name) {
            Ok(mut branch_reference) => {
                let name = match branch_reference.name() {
                    Some(s) => s.to_string(),
                    None => String::from_utf8_lossy(branch_reference.name_bytes()).to_string(),
                };
                let message = format!("Fast Forward: Setting {} to id: {}", name, commit.id());
                branch_reference.set_target(commit.id(), &message)?;
                repo.set_head(&name)?;
                repo.checkout_head(Some(CheckoutBuilder::default().force()))?;
                Self::current_merge_status(repo)
            }
            Err(_) => {
                repo.reference(
                    &branch_reference_name,
                    commit.id(),
                    true,
                    &format!("Setting {} to {}", branch_name, commit.id()),
                )?;
                repo.set_head(&branch_reference_name)?;
                repo.checkout_head(Some(
                    CheckoutBuilder::default()
                        .allow_conflicts(true)
                        .conflict_style_merge(true)
                        .force(),
                ))?;
                Self::current_merge_status(repo)
            }
        }
    }

    fn current_merge_status(repo: &Repository) -> Result<MergeBranchStatus> {
        if repo.index()?.has_conflicts() {
            Ok(MergeBranchStatus::MergeConflict)
        } else {
            Ok(MergeBranchStatus::Success)
        }
    }

    fn commit_all(repo: &Repository, message: &str) -> Result<()> {
        let mut index = repo.index()?;
        index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;
        index.write()?;
        let oid = index.write_tree()?;
        let signature = repo.signature()?;
        let parent_commit = repo.head()?.peel_to_commit()?;
        let tree = repo.find_tree(oid)?;
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[&parent_commit],
        )?;
        Ok(())
    }

    fn checkout_new_branch(repo: &Repository, name: &RoswaalOwnedGitBranchName) -> Result<()> {
        let commit = repo.head()?.peel_to_commit()?;
        _ = repo.branch(&name.to_string(), &commit, false)?;
        let (object, reference) = repo.revparse_ext(&name.to_string())?;
        repo.checkout_tree(&object, None)?;
        if let Some(name) = reference.and_then(|r| r.name().map(|s| s.to_string())) {
            repo.set_head(&name)?;
        }
        Ok(())
    }

    fn push_changes(
        repo: &Repository,
        branch_name: &RoswaalOwnedGitBranchName,
        callbacks: RemoteCallbacks,
    ) -> Result<()> {
        let mut remote = repo.find_remote("origin")?;
        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(callbacks);
        remote.push(
            &[format!("refs/heads/{}", branch_name.to_string())],
            Some(&mut push_options),
        )?;
        Ok(())
    }

    fn delete_local_branch(
        repo: &Repository,
        branch_name: &RoswaalOwnedGitBranchName,
    ) -> Result<bool> {
        let branch = repo
            .branches(Some(BranchType::Local))?
            .filter_map(|branch_result| branch_result.ok().map(|(branch, _)| branch))
            .find(|branch| {
                branch
                    .name()
                    .ok()
                    .map(|name| name == Some(&branch_name.to_string()))
                    .unwrap_or(false)
            });
        if let Some(mut branch) = branch {
            branch.delete()?;
            return Ok(true);
        } else {
            Ok(false)
        }
    }

    fn statuses(repo: &Repository) -> Result<Vec<LibGit2StatusEntry>> {
        let statuses = repo
            .statuses(None)?
            .iter()
            .filter_map(|entry| {
                if let (Some(entry_path), true) = (entry.path(), entry.status().is_wt_new()) {
                    Some(LibGit2StatusEntry {
                        path: repo.workdir().unwrap().join(entry_path),
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<LibGit2StatusEntry>>();
        Ok(statuses)
    }
}

impl RoswaalGitRepositoryMetadata {
    fn remote_callbacks(&self) -> RemoteCallbacks {
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_, user, _| {
            let path = self.ssh_private_key_path();
            Cred::ssh_key(user.unwrap(), None, Path::new(&path), None)
        });
        callbacks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::fs::{create_dir_all, try_exists, File};

    use crate::git::{
        branch_name::RoswaalOwnedGitBranchName,
        test_support::{
            read_string, repo_with_test_metadata, with_clean_test_repo_access, write_string,
        },
    };

    #[tokio::test]
    async fn test_add_commit_push_pull() {
        with_clean_test_repo_access(async {
            let (repo, metadata) = repo_with_test_metadata().await?;
            let transaction = repo.transaction().await;
            let branch_name = RoswaalOwnedGitBranchName::new("test");
            transaction.checkout_new_branch(&branch_name).await?;

            let expected_file_contents =
                "In this world, all life will walk towards the future, hand in hand.";
            write_string(&metadata.relative_path("test.txt"), expected_file_contents).await?;

            transaction.commit_all("I like this!").await?;
            transaction.push_changes(&branch_name).await?;
            transaction
                .switch_branch(metadata.base_branch_name())
                .await?;

            assert!(!try_exists(metadata.relative_path("test.txt")).await?);

            let status = transaction.pull_branch(&branch_name.to_string()).await?;
            assert_eq!(status, PullBranchStatus::Success);

            let contents = read_string(&metadata.relative_path("test.txt")).await?;
            assert_eq!(contents, expected_file_contents);
            Ok(())
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_reset_hard_to_head() {
        with_clean_test_repo_access(async {
            let (repo, metadata) = repo_with_test_metadata().await?;
            let transaction = repo.transaction().await;

            write_string(metadata.locations_path(), "console.log(\"Hello world\")").await?;

            transaction.hard_reset_to_head().await?;

            assert!(read_string(metadata.locations_path()).await?.is_empty());
            Ok(())
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_clean_all_untracked() {
        with_clean_test_repo_access(async {
            let (repo, metadata) = repo_with_test_metadata().await?;
            let transaction = repo.transaction().await;

            File::create(metadata.relative_path("test.txt")).await?;
            create_dir_all(metadata.relative_path("roswaal/nested/test-clean")).await?;
            File::create(metadata.relative_path("roswaal/nested/test-clean/test2.txt")).await?;

            transaction.clean_all_untracked().await?;

            assert!(!try_exists(metadata.relative_path("test.txt")).await?);
            assert!(
                !try_exists(metadata.relative_path("roswaal/nested/test-clean/test2.txt")).await?
            );
            assert!(!try_exists(metadata.relative_path("roswaal/nested/test-clean")).await?);
            assert!(!try_exists(metadata.relative_path("roswaal/nested")).await?);
            assert!(try_exists(metadata.relative_path("roswaal")).await?);
            Ok(())
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_delete_local_branch_that_exists_returns_true_when_deleted_properly() {
        with_clean_test_repo_access(async {
            let (repo, _) = repo_with_test_metadata().await?;
            let transaction = repo.transaction().await;

            let branch_name = RoswaalOwnedGitBranchName::new("test");
            transaction.checkout_new_branch(&branch_name).await?;
            transaction.switch_branch("main").await?;
            let did_remove = transaction.delete_local_branch(&branch_name).await?;
            assert!(did_remove);
            let switch_to_deleted = transaction.switch_branch(&branch_name.to_string()).await;
            assert!(switch_to_deleted.is_err());

            Ok(())
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_delete_local_branch_returns_false_for_non_existent_branch() {
        with_clean_test_repo_access(async {
            let (repo, _) = repo_with_test_metadata().await?;
            let transaction = repo.transaction().await;

            let branch_name = RoswaalOwnedGitBranchName::new("test");
            let did_remove = transaction.delete_local_branch(&branch_name).await?;
            assert!(!did_remove);

            Ok(())
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_pull_merge_conflict() {
        with_clean_test_repo_access(async {
            let (repo, metadata) = repo_with_test_metadata().await?;
            let transaction = repo.transaction().await;

            let b1 = RoswaalOwnedGitBranchName::new("test");
            let b2 = RoswaalOwnedGitBranchName::new("test2");

            transaction.checkout_new_branch(&b1).await?;

            write_string(metadata.locations_path(), "console.log(\"Hello world\")").await?;

            transaction.commit_all("console.log").await?;
            transaction
                .switch_branch(metadata.base_branch_name())
                .await?;
            transaction.checkout_new_branch(&b2).await?;

            write_string(metadata.locations_path(), "console.log(\"Goodbye world\")").await?;

            transaction.commit_all("console.log").await?;
            transaction.push_changes(&b2).await?;
            transaction.switch_branch(&b1.to_string()).await?;

            let status = transaction.pull_branch(&b2.to_string()).await?;
            assert_eq!(status, PullBranchStatus::MergeConflict);

            Ok(())
        })
        .await
        .unwrap();
    }
}
