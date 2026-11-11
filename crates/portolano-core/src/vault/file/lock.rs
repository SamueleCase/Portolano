//! Inter-process lock on the vault file (DEC-017).
//!
//! The app and the CLI may have the same vault open at once. Every operation
//! takes the lock on `<vault>.lock` for its own duration only: readers share
//! it, a read-modify-write holds it exclusively. The lock file is never
//! deleted, because removing it while another process waits on it would let
//! two writers in at once.

use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

use fd_lock::RwLock;

use crate::error::VaultError;

/// How the vault is accessed while the lock is held.
#[derive(Debug, Clone, Copy)]
pub(super) enum Access {
    /// Reading only: other readers may run at the same time.
    Shared,
    /// Reading and rewriting: no other process may read or write.
    Exclusive,
}

/// Runs `operation` while holding the lock of the vault at `vault_path`.
///
/// # Errors
/// [`VaultError::Io`] on the lock file, or the error of `operation`.
pub(super) fn with_lock<T>(
    vault_path: &Path,
    access: Access,
    operation: impl FnOnce() -> Result<T, VaultError>,
) -> Result<T, VaultError> {
    let lock_path = lock_path(vault_path);
    let io_error = |source| VaultError::Io {
        path: lock_path.clone(),
        source,
    };
    let mut lock = RwLock::new(open_lock_file(&lock_path).map_err(io_error)?);
    match access {
        Access::Shared => {
            let _guard = lock.read().map_err(io_error)?;
            operation()
        }
        Access::Exclusive => {
            let _guard = lock.write().map_err(io_error)?;
            operation()
        }
    }
}

/// `vault.json` → `vault.json.lock`, in the same directory.
fn lock_path(vault_path: &Path) -> PathBuf {
    let mut name = OsString::from(vault_path.as_os_str());
    name.push(".lock");
    PathBuf::from(name)
}

fn open_lock_file(lock_path: &Path) -> io::Result<File> {
    if let Some(dir) = lock_path.parent().filter(|dir| !dir.as_os_str().is_empty()) {
        fs::create_dir_all(dir)?;
    }
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true).truncate(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(lock_path)
}
