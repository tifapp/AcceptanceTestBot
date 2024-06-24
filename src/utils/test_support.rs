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

#[cfg(test)]
pub async fn with_test_repo_access(work: impl Future<Output = Result<()>>) -> Result<()> {
    _ = LOCK.lock().await;
    Command::new("./setup_test_repo.sh").spawn()?.wait().await?;
    work.await
}
