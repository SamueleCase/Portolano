//! The advisory lock of a document, `<file>.lock`, held only for the length
//! of one operation (DEC-016.1).

use std::fs::OpenOptions;
use std::path::Path;

use super::{io_error, with_suffix};
use crate::error::Result;

/// How a document operation locks the document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Access {
    /// Readers exclude writers but not each other.
    Shared,
    /// Writers exclude everybody.
    Exclusive,
}

/// Runs `operation` while holding the lock of `document` with `access`,
/// waiting for other processes to release it. Operations are short, so the
/// wait is too. The lock file is left on disk: removing it would let another
/// process lock a file that is about to disappear.
pub(crate) fn with_lock<T>(
    document: &Path,
    access: Access,
    operation: impl FnOnce() -> Result<T>,
) -> Result<T> {
    let lock_path = with_suffix(document, ".lock");
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&lock_path)
        .map_err(io_error(&lock_path))?;
    let mut lock = fd_lock::RwLock::new(file);
    // Each guard lives until the end of its branch, so the lock covers the
    // whole operation and is released before returning.
    match access {
        Access::Shared => {
            let _guard = lock.read().map_err(io_error(&lock_path))?;
            operation()
        }
        Access::Exclusive => {
            let _guard = lock.write().map_err(io_error(&lock_path))?;
            operation()
        }
    }
}
