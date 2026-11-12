//! Size and position of the main window, remembered in `ui.toml`
//! (SPEC-UI §4.9).

use tauri::{LogicalPosition, LogicalSize, Manager, Runtime, WebviewWindow};

use crate::prefs::WindowGeometry;
use crate::state::AppState;

/// The label of the main window (`tauri.conf.json`).
pub const MAIN: &str = "main";

/// Gives the main window the size and position it had when the app last
/// closed.
pub fn restore<R: Runtime>(window: &WebviewWindow<R>, geometry: WindowGeometry) {
    let placed = window
        .set_size(LogicalSize::new(geometry.width, geometry.height))
        .and_then(|()| window.set_position(LogicalPosition::new(geometry.x, geometry.y)));
    if let Err(error) = placed {
        tracing::warn!(%error, "dimensioni della finestra non ripristinate");
    }
}

/// Remembers the size and position of the main window.
pub fn remember<R: Runtime>(window: &WebviewWindow<R>) {
    let Some(geometry) = geometry_of(window) else {
        return;
    };
    let state = window.state::<AppState>();
    if let Err(error) =
        state.update_ui_preferences(|preferences| preferences.window = Some(geometry))
    {
        tracing::warn!("{}", error.message);
    }
}

fn geometry_of<R: Runtime>(window: &WebviewWindow<R>) -> Option<WindowGeometry> {
    let scale = window.scale_factor().ok()?;
    let size = window.outer_size().ok()?.to_logical::<f64>(scale);
    let position = window.outer_position().ok()?.to_logical::<f64>(scale);
    Some(WindowGeometry {
        width: size.width,
        height: size.height,
        x: position.x,
        y: position.y,
    })
}
