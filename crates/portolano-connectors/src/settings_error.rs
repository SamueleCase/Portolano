//! The settings of a connection that its protocol refuses, with where the
//! problem is as data (DEC-040.3): an interface shows the message next to
//! the setting without reading the text.

/// Settings refused by a protocol. Shown, it is the message alone, the same
/// text the CLI prints.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{message}")]
pub struct SettingsError {
    /// The top-level key of the settings at fault, such as `options` or
    /// `jump_hosts`; none when the problem is not in a single key.
    pub setting: Option<String>,
    /// The ssh option refused, inside `options`.
    pub option: Option<String>,
    /// What is wrong, in Italian.
    pub message: String,
}

impl SettingsError {
    /// A problem of the settings as a whole.
    pub(crate) fn whole(message: String) -> Self {
        Self {
            setting: None,
            option: None,
            message,
        }
    }

    /// A problem of the setting `key`.
    pub(crate) fn at(key: &str, message: String) -> Self {
        Self {
            setting: Some(key.to_owned()),
            option: None,
            message,
        }
    }

    /// A refused ssh option `key`, inside `options`.
    pub(crate) fn option(key: &str, message: String) -> Self {
        Self {
            setting: Some("options".to_owned()),
            option: Some(key.to_owned()),
            message,
        }
    }
}
