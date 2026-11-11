//! Atomic replacement of the vault file.

use std::fs;
use std::io::{self, Write};
use std::path::Path;

/// Whether the vault file may already exist.
#[derive(Debug, Clone, Copy)]
pub(super) enum WriteMode {
    /// The file must not exist: a vault created concurrently by another
    /// process is never overwritten.
    CreateNew,
    /// The file is replaced.
    Replace,
}

/// Writes `contents` to a temporary file in the same directory, flushes it to
/// disk with owner-only permissions and renames it to `path`, so that a crash
/// leaves either the old or the new vault, never a partial one.
pub(super) fn write_atomically(path: &Path, contents: &[u8], mode: WriteMode) -> io::Result<()> {
    let dir = match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    };
    fs::create_dir_all(dir)?;
    let mut temp = tempfile::Builder::new()
        .prefix(".vault-")
        .suffix(".tmp")
        .tempfile_in(dir)?;
    restrict_to_owner(temp.as_file())?;
    temp.write_all(contents)?;
    temp.as_file().sync_all()?;
    match mode {
        WriteMode::CreateNew => temp.persist_noclobber(path),
        WriteMode::Replace => temp.persist(path),
    }
    .map_err(|error| error.error)?;
    sync_directory(dir)
}

#[cfg(unix)]
fn restrict_to_owner(file: &fs::File) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    // Set explicitly instead of relying on the umask or on tempfile defaults.
    file.set_permissions(fs::Permissions::from_mode(0o600))
}

#[cfg(not(unix))]
fn restrict_to_owner(_file: &fs::File) -> io::Result<()> {
    Ok(())
}

/// Makes the rename itself durable.
#[cfg(unix)]
fn sync_directory(dir: &Path) -> io::Result<()> {
    fs::File::open(dir)?.sync_all()
}

#[cfg(not(unix))]
fn sync_directory(_dir: &Path) -> io::Result<()> {
    Ok(())
}
