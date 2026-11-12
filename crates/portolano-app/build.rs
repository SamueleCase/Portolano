//! Build script: Tauri context, and the permissions of the app commands.

/// The commands of the app. Listing them makes Tauri generate one
/// `allow-<command>` permission each, so that a window gets only the
/// commands its capability names (SPEC-UI §4.8). A command registered in
/// `lib.rs` but missing here is refused at runtime.
const COMMANDS: &[&str] = &["app_status"];

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let manifest = tauri_build::AppManifest::new().commands(COMMANDS);
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(manifest))?;
    Ok(())
}
