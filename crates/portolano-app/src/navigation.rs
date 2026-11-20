//! The navigation guard of the webviews (SPEC-UI §4.8, DEC-046.2): a page
//! may only ever show the app itself. A link, a script or a form that
//! tries to load anything else (a remote page, a `portolano-icon://` image
//! as a document, a `file://` path) is refused before the webview loads it,
//! so the commands of the app are never reachable from another origin.

use tauri::plugin::{Builder, TauriPlugin};
use tauri::{Manager, Runtime, Url};

/// The plugin that guards every navigation of the webviews of the app.
pub fn guard<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("navigation-guard")
        .on_navigation(|webview, url| {
            // The page of `tauri dev` comes from Vite, only in development.
            let dev_url = if tauri::is_dev() {
                webview.app_handle().config().build.dev_url.clone()
            } else {
                None
            };
            let allowed = is_app_page(url, dev_url.as_ref());
            if !allowed {
                // Only the scheme: the address may come from the document.
                tracing::warn!("navigazione rifiutata verso un indirizzo {}:", url.scheme());
            }
            allowed
        })
        .build()
}

/// Whether `url` is a page of the app: its own origin, `tauri://localhost`
/// (`http://tauri.localhost` on Windows), or the origin of `dev_url` while
/// developing.
pub(crate) fn is_app_page(url: &Url, dev_url: Option<&Url>) -> bool {
    let bundled = is_origin(url, "tauri", "localhost") || is_windows_origin(url);
    bundled || dev_url.is_some_and(|dev| url.origin() == dev.origin())
}

/// Whether `url` has `scheme`, `host` and no port of its own.
fn is_origin(url: &Url, scheme: &str, host: &str) -> bool {
    url.scheme() == scheme && url.host_str() == Some(host) && url.port().is_none()
}

/// The origin under which Tauri serves the app on Windows (DEC-046.9): the
/// workspace builds there too (SPEC-UI §10).
#[cfg(windows)]
fn is_windows_origin(url: &Url) -> bool {
    is_origin(url, "http", "tauri.localhost")
}

/// Elsewhere the app is never served from an `http` origin.
#[cfg(not(windows))]
fn is_windows_origin(_url: &Url) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url(text: &str) -> Url {
        Url::parse(text).unwrap()
    }

    #[test]
    fn admits_the_pages_of_the_app() {
        assert!(is_app_page(&url("tauri://localhost"), None));
        assert!(is_app_page(
            &url("tauri://localhost/index.html?gallery"),
            None
        ));
    }

    #[test]
    fn refuses_a_remote_page() {
        assert!(!is_app_page(&url("https://attacker.example.com/"), None));
        assert!(!is_app_page(&url("http://localhost:5173/"), None));
    }

    #[test]
    fn refuses_the_icons_as_a_page() {
        assert!(!is_app_page(
            &url("portolano-icon://localhost/aa.svg"),
            None
        ));
    }

    #[test]
    fn refuses_local_files_and_other_hosts_of_the_scheme() {
        assert!(!is_app_page(&url("file:///etc/passwd"), None));
        assert!(!is_app_page(&url("tauri://localhost.example.com/"), None));
        assert!(!is_app_page(&url("tauri://localhost:8080/"), None));
        assert!(!is_app_page(&url("data:text/html,<p>x</p>"), None));
    }

    #[cfg(not(windows))]
    #[test]
    fn refuses_the_origin_of_windows_elsewhere() {
        assert!(!is_app_page(&url("http://tauri.localhost/"), None));
    }

    #[cfg(windows)]
    #[test]
    fn admits_the_origin_of_the_app_on_windows() {
        assert!(is_app_page(&url("http://tauri.localhost/index.html"), None));
        assert!(!is_app_page(&url("http://tauri.localhost:8080/"), None));
        assert!(!is_app_page(&url("https://tauri.localhost/"), None));
    }

    #[test]
    fn admits_the_page_of_vite_only_as_the_dev_url() {
        let dev = url("http://localhost:5173");

        assert!(is_app_page(
            &url("http://localhost:5173/?gallery"),
            Some(&dev)
        ));
        assert!(!is_app_page(&url("http://localhost:5174/"), Some(&dev)));
        assert!(!is_app_page(&url("https://localhost:5173/"), Some(&dev)));
    }
}
