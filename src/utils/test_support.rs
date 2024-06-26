#[cfg(test)]
use std::{future::Future, sync::Arc};
#[cfg(test)]
use anyhow::Result;
#[cfg(test)]
use once_cell::sync::Lazy;
#[cfg(test)]
use tokio::process::Command;
#[cfg(test)]
use tokio::sync::Mutex;

#[cfg(test)]
static LOCK: Lazy<Arc<Mutex<()>>> = Lazy::new(|| Arc::new(Mutex::new(())));

/// Cleans and serializes access to the test repo for the duration of the future.
#[cfg(test)]
pub async fn with_clean_test_repo_access(work: impl Future<Output = Result<()>>) -> Result<()> {
    let guard = LOCK.lock().await;
    Command::new("./reset_test_repo.sh").spawn()?.wait().await?;
    let result = work.await;
    drop(guard);
    result
}
