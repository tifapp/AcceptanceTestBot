#[cfg(test)]
use anyhow::Result;
#[cfg(test)]
use tokio::process::Command;

/// Resets the test repo for testing purposes.
#[cfg(test)]
pub async fn reset_test_repo() -> Result<()> {
    Command::new("./setup_test_repo.sh").spawn()?.wait().await?;
    Ok(())
}
