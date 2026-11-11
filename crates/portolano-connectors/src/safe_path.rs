//! Checks on the client programs named in the configuration, like
//! `safe_path()` of OpenSSH (DEC-024.2, DEC-026.3): such a program receives
//! the passwords, so nobody but the user (or root) may be able to replace
//! it, nor any directory on its path.

use std::fs;
use std::path::Path;

/// Checks that the program configured as `key` is an absolute path, without
/// touching the filesystem, as [`Protocol::validate_config`] requires.
///
/// [`Protocol::validate_config`]: crate::Protocol::validate_config
///
/// # Errors
/// A description of the problem, in Italian.
pub(crate) fn check_program_path(key: &str, program: &Path) -> Result<(), String> {
    if program.is_absolute() {
        Ok(())
    } else {
        Err(format!(
            "'{key}' deve essere un percorso assoluto, non '{}'",
            program.display()
        ))
    }
}

/// Checks that the program configured as `key` is an absolute path to a
/// regular, executable file; that the file and every directory of its path,
/// both as written and after resolving the symbolic links, belong to the
/// user or to root; and that none of them is writable by the group or by
/// others.
///
/// # Errors
/// A description of the problem, in Italian, naming the element at fault.
pub(crate) fn check_trusted_program(key: &str, program: &Path) -> Result<(), String> {
    check_program_path(key, program)?;
    let shown = program.display();
    let resolved = fs::canonicalize(program)
        .map_err(|_| format!("'{key}': '{shown}' non esiste o non è accessibile"))?;
    let metadata = fs::metadata(&resolved)
        .map_err(|_| format!("'{key}': '{shown}' non esiste o non è accessibile"))?;
    if !metadata.is_file() {
        return Err(format!("'{key}': '{shown}' non è un file regolare"));
    }
    #[cfg(unix)]
    {
        let uid = current_uid()
            .map_err(|_| format!("'{key}': impossibile determinare l'utente corrente"))?;
        check_owners_and_modes(key, program, &resolved, uid)?;
    }
    Ok(())
}

/// The checks of [`check_trusted_program`] that depend on the owners and
/// the modes, for the user `uid`.
#[cfg(unix)]
fn check_owners_and_modes(
    key: &str,
    program: &Path,
    resolved: &Path,
    uid: u32,
) -> Result<(), String> {
    use std::os::unix::fs::MetadataExt;

    let shown = program.display();
    // The directories of the path as written hold the links that lead to
    // the program: whoever could write there could point them elsewhere.
    let written = program.parent().into_iter().flat_map(Path::ancestors);
    for element in resolved.ancestors().chain(written) {
        let refuse = |reason: &str| {
            let place = if element == program {
                String::new()
            } else {
                format!(": '{}'", element.display())
            };
            Err(format!("'{key}' ('{shown}'){place} {reason}"))
        };
        let Ok(metadata) = fs::metadata(element) else {
            return refuse("non è accessibile");
        };
        if metadata.uid() != uid && metadata.uid() != 0 {
            return refuse("appartiene a un altro utente");
        }
        if metadata.mode() & 0o022 != 0 {
            return refuse("è scrivibile dal gruppo o da altri: togli quel permesso (chmod go-w)");
        }
        if element == resolved && metadata.mode() & 0o111 == 0 {
            return refuse("non è eseguibile");
        }
    }
    Ok(())
}

/// The effective user id. The owner of a new anonymous file is that user,
/// which avoids an `unsafe` call to `getuid` (the crate forbids `unsafe`).
#[cfg(unix)]
pub(crate) fn current_uid() -> std::io::Result<u32> {
    use std::os::unix::fs::MetadataExt;
    Ok(tempfile::tempfile()?.metadata()?.uid())
}

#[cfg(test)]
#[cfg(unix)]
mod tests {
    use std::os::unix::fs::{PermissionsExt, symlink};
    use std::path::PathBuf;

    use pretty_assertions::assert_eq;

    use super::*;

    fn program(dir: &Path, name: &str, mode: u32) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, "#!/bin/sh\n").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(mode)).unwrap();
        path
    }

    fn directory(parent: &Path, name: &str, mode: u32) -> PathBuf {
        let path = parent.join(name);
        fs::create_dir(&path).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(mode)).unwrap();
        path
    }

    #[test]
    fn a_private_executable_is_trusted() {
        let dir = tempfile::tempdir().unwrap();
        let ssh = program(dir.path(), "my-ssh", 0o755);

        assert_eq!(check_trusted_program("program", &ssh), Ok(()));
    }

    #[test]
    fn the_program_must_be_absolute() {
        let error = check_trusted_program("program", Path::new("bin/ssh")).unwrap_err();

        assert!(error.contains("percorso assoluto"), "{error}");
    }

    #[test]
    fn the_program_must_exist() {
        let dir = tempfile::tempdir().unwrap();

        let error = check_trusted_program("program", &dir.path().join("missing")).unwrap_err();

        assert!(error.contains("non esiste"), "{error}");
    }

    #[test]
    fn the_program_must_be_a_regular_file() {
        let dir = tempfile::tempdir().unwrap();

        let error = check_trusted_program("program", dir.path()).unwrap_err();

        assert!(error.contains("non è un file regolare"), "{error}");
    }

    #[test]
    fn the_program_must_not_be_writable_by_the_group_or_others() {
        let dir = tempfile::tempdir().unwrap();
        for mode in [0o775, 0o757] {
            let ssh = program(dir.path(), &format!("ssh-{mode:o}"), mode);

            let error = check_trusted_program("program", &ssh).unwrap_err();

            assert!(error.contains("chmod go-w"), "{error}");
        }
    }

    #[test]
    fn the_program_must_be_executable() {
        let dir = tempfile::tempdir().unwrap();
        let ssh = program(dir.path(), "ssh", 0o644);

        let error = check_trusted_program("program", &ssh).unwrap_err();

        assert!(error.contains("non è eseguibile"), "{error}");
    }

    #[test]
    fn a_directory_writable_by_the_group_is_named() {
        let dir = tempfile::tempdir().unwrap();
        let shared = directory(dir.path(), "shared", 0o775);
        let ssh = program(&shared, "ssh", 0o755);

        let error = check_trusted_program("program", &ssh).unwrap_err();

        assert!(error.contains("shared'"), "{error}");
        assert!(
            error.contains("scrivibile dal gruppo o da altri"),
            "{error}"
        );
    }

    #[test]
    fn a_path_owned_by_another_user_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let ssh = program(dir.path(), "ssh", 0o755);
        let resolved = fs::canonicalize(&ssh).unwrap();
        // Simulated without privileges: the files belong to the real user,
        // the check runs as someone else (neither the user nor root).
        let other = current_uid().unwrap() + 1;

        let error = check_owners_and_modes("program", &ssh, &resolved, other).unwrap_err();

        assert!(error.contains("appartiene a un altro utente"), "{error}");
    }

    #[test]
    fn a_link_to_a_program_in_a_writable_directory_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let shared = directory(dir.path(), "shared", 0o777);
        let target = program(&shared, "ssh", 0o755);
        let link = dir.path().join("ssh-link");
        symlink(&target, &link).unwrap();

        let error = check_trusted_program("program", &link).unwrap_err();

        assert!(error.contains("shared'"), "{error}");
    }

    #[test]
    fn a_link_in_a_writable_directory_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let target = program(dir.path(), "ssh", 0o755);
        let shared = directory(dir.path(), "shared", 0o777);
        let link = shared.join("ssh-link");
        symlink(&target, &link).unwrap();

        let error = check_trusted_program("program", &link).unwrap_err();

        assert!(error.contains("shared'"), "{error}");
    }

    #[test]
    fn a_link_to_a_trusted_program_is_trusted() {
        let dir = tempfile::tempdir().unwrap();
        let target = program(dir.path(), "ssh", 0o755);
        let link = dir.path().join("ssh-link");
        symlink(&target, &link).unwrap();

        assert_eq!(check_trusted_program("program", &link), Ok(()));
    }
}
