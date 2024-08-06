use std::{
    fs::{read_dir, remove_dir},
    io::{self, Result},
    path::Path,
};

use tokio::task::spawn_blocking;

/// Recursively removes all empty directories at the specified path.
pub async fn remove_dir_all_empty(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref().to_owned();
    asyncify(move || remove_dir_all_empty_sync(path)).await
}

fn remove_dir_all_empty_sync(path: impl AsRef<Path>) -> Result<()> {
    if !path.as_ref().is_dir() {
        return Ok(());
    }
    let mut dir = read_dir(path.as_ref().to_owned())?;
    let mut next_entry = dir.next();
    while let Some(entry) = next_entry.and_then(|e| e.ok()) {
        if entry.path().is_dir() {
            remove_dir_all_empty_sync(entry.path())?;
        }
        next_entry = dir.next();
    }
    dir = read_dir(path.as_ref().to_owned())?;
    if dir.next().is_none() {
        remove_dir(path)?;
        return Ok(());
    }
    Ok(())
}

// NB: Copied from tokio::fs.
async fn asyncify<F, T>(f: F) -> io::Result<T>
where
    F: FnOnce() -> io::Result<T> + Send + 'static,
    T: Send + 'static,
{
    match spawn_blocking(f).await {
        Ok(res) => res,
        Err(_) => Err(io::Error::new(
            io::ErrorKind::Other,
            "background task failed",
        )),
    }
}
