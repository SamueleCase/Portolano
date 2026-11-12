//! Icons as the frontend shows them (SPEC-UI §4.7).

use portolano_core::Icon;
use portolano_core::icons::is_sha256_hex;
use serde::Serialize;

use crate::builtin_icons;
use crate::icons::SCHEME;

/// An icon to show. A node without one shows the icon of its type: so does
/// a Royal icon without a builtin match, or a builtin name this version does
/// not know (a document written by hand or by a newer version).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum IconView {
    /// A builtin icon, by name.
    Builtin {
        /// One of the builtin names.
        name: String,
    },
    /// A custom image of the document, shown with `<img src={url}>`.
    Custom {
        /// The hash of the image, to choose it again in the editor.
        sha256: String,
        /// Its type.
        mime: String,
        /// Its URL on the `portolano-icon` scheme.
        url: String,
    },
}

impl IconView {
    /// The view of `icon`, if it has one.
    pub fn of(icon: &Icon) -> Option<IconView> {
        match icon {
            Icon::Builtin { name } => {
                builtin_icons::is_builtin(name).then(|| IconView::Builtin { name: name.clone() })
            }
            Icon::Royal { name } => builtin_icons::for_royal(name).map(|name| IconView::Builtin {
                name: name.to_owned(),
            }),
            Icon::Custom { sha256, mime } => {
                let extension = extension_of(mime)?;
                // A hand-edited document may hold anything: only a hash can
                // become a URL.
                is_sha256_hex(sha256).then(|| IconView::Custom {
                    sha256: sha256.clone(),
                    mime: mime.clone(),
                    url: icon_url(&format!("{sha256}.{extension}")),
                })
            }
        }
    }
}

/// The extension of the stored images of type `mime`.
fn extension_of(mime: &str) -> Option<&'static str> {
    ["png", "jpg", "gif", "tiff", "svg", "pdf"]
        .into_iter()
        .find(|extension| portolano_core::icons::mime_for_extension(extension) == Some(mime))
}

/// The URL of the stored image `file_name`: WebKit (macOS, Linux) loads
/// custom schemes as `<scheme>://localhost/`, WebView2 (Windows) as
/// `http://<scheme>.localhost/`.
fn icon_url(file_name: &str) -> String {
    if cfg!(windows) {
        format!("http://{SCHEME}.localhost/{file_name}")
    } else {
        format!("{SCHEME}://localhost/{file_name}")
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn royal_icon_is_shown_with_its_builtin_match() {
        let icon = Icon::Royal {
            name: "Flat/Hardware/Server".to_owned(),
        };

        assert_eq!(
            IconView::of(&icon),
            Some(IconView::Builtin {
                name: "server".to_owned()
            })
        );
    }

    #[test]
    fn unknown_builtin_name_shows_the_icon_of_the_type() {
        let icon = Icon::Builtin {
            name: "astronave".to_owned(),
        };

        assert_eq!(IconView::of(&icon), None);
    }

    #[test]
    fn custom_image_gets_its_url_on_the_icon_scheme() {
        let sha256 = "ab".repeat(32);
        let icon = Icon::Custom {
            sha256: sha256.clone(),
            mime: "image/svg+xml".to_owned(),
        };

        let Some(IconView::Custom { url, .. }) = IconView::of(&icon) else {
            panic!("custom icon expected");
        };

        assert!(url.ends_with(&format!("/{sha256}.svg")), "{url}");
        assert!(url.contains(SCHEME));
    }

    #[test]
    fn custom_image_with_a_malformed_hash_is_not_shown() {
        let icon = Icon::Custom {
            sha256: "../../segreti".to_owned(),
            mime: "image/png".to_owned(),
        };

        assert_eq!(IconView::of(&icon), None);
    }
}
