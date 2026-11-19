//! The native dialogs (SPEC-UI §4.6): opened by Rust. The frontend only
//! learns the name of the file chosen, or that a change was not confirmed.

use portolano_core::text::visible;
use tauri::{AppHandle, Runtime};
use tauri_plugin_dialog::{
    DialogExt, FileDialogBuilder, FilePath, MessageDialogButtons, MessageDialogKind,
};

use crate::dto::ChosenFile;
use crate::state::{AppState, FileKind};

/// A dialog for Portolano documents (`*.json`, ambiguity 16).
pub fn document<R: Runtime>(app: &AppHandle<R>) -> FileDialogBuilder<R> {
    app.dialog()
        .file()
        .add_filter("Documento Portolano", &["json"])
}

/// A dialog for documents of Royal TS/TSX: `.rtsz`, `.rtsx`, XML, also
/// compressed (the importer recognizes the content, not the extension).
pub fn import<R: Runtime>(app: &AppHandle<R>) -> FileDialogBuilder<R> {
    app.dialog()
        .file()
        .add_filter("Documento Royal TS", &["rtsz", "rtsx", "xml", "gz"])
}

/// A dialog for icon images, of the types the icon store accepts.
pub fn icon<R: Runtime>(app: &AppHandle<R>) -> FileDialogBuilder<R> {
    app.dialog().file().add_filter(
        "Immagine",
        &["png", "jpg", "jpeg", "gif", "tif", "tiff", "svg", "pdf"],
    )
}

/// A dialog for the JSON report of an import.
pub fn report<R: Runtime>(app: &AppHandle<R>) -> FileDialogBuilder<R> {
    app.dialog()
        .file()
        .add_filter("Report JSON", &["json"])
        .set_file_name("report-import.json")
}

/// A dialog for a private key file, starting from `~/.ssh` if it exists.
pub fn key_file<R: Runtime>(app: &AppHandle<R>) -> FileDialogBuilder<R> {
    let dialog = app.dialog().file();
    match std::env::home_dir().map(|home| home.join(".ssh")) {
        Some(ssh) if ssh.is_dir() => dialog.set_directory(ssh),
        _ => dialog,
    }
}

/// A dialog for any file: a program or the file vault.
pub fn any_file<R: Runtime>(app: &AppHandle<R>) -> FileDialogBuilder<R> {
    app.dialog().file()
}

/// Remembers the file chosen for `kind`, if the user chose one, and returns
/// its name for the frontend.
pub fn remember(state: &AppState, kind: FileKind, chosen: Option<FilePath>) -> Option<ChosenFile> {
    let path = local_path(chosen)?;
    state.chosen().set(kind, &path);
    Some(ChosenFile::of(&path))
}

/// Asks, with a native dialog of macOS, whether `command` is the custom
/// terminal command the user wants: Portolano runs it with the shell for
/// every new window (DEC-046.1). The webview can ask for it, never answer.
/// The command shows as it is, its control and bidi characters as escapes
/// ([`visible`], the rule of the CLI).
pub fn confirm_terminal_command<R: Runtime>(app: &AppHandle<R>, command: &str) -> bool {
    app.dialog()
        .message(format!(
            "Portolano eseguirà questo comando con la shell a ogni «Apri in terminale esterno»:\n\n\
             {}\n\nUsalo solo se l'hai scritto tu.",
            visible(command)
        ))
        .title("Usare questo comando del terminale?")
        .kind(MessageDialogKind::Warning)
        .buttons(MessageDialogButtons::OkCancelCustom(
            "Usa questo comando".to_owned(),
            "Annulla".to_owned(),
        ))
        .blocking_show()
}

/// The local path of a dialog answer; `None` if the user cancelled.
pub fn local_path(chosen: Option<FilePath>) -> Option<std::path::PathBuf> {
    chosen?.into_path().ok()
}
