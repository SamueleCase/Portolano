//! Recognition of the OpenSSH prompts that askpass can answer (SPEC §7.2
//! point 4, DEC-015).

use regex::Regex;

use crate::plan::PromptMatcher;

// A password is answered only when the user and the host in the prompt
// equal those of its entry. A malicious server cannot obtain the password of
// another host by printing that host's prompt: OpenSSH writes the prompt
// itself from the user and host it connects to (`user@host's password: `),
// and since 8.4 it also prefixes the keyboard-interactive prompts chosen by
// the server with `(user@host) `, so a server only ever sees its own prompts
// answered. Any other text gets a fallback to the terminal.

/// Password prompt of `ssh`: `user@host's password: `.
const PASSWORD: &str = r"^(?P<user>[^@\s]+)@(?P<host>[^']+)'s password: ?$";

/// Keyboard-interactive password prompt of `ssh`: `(user@host) Password: `.
const KEYBOARD_INTERACTIVE: &str = r"^\((?P<user>[^@\s]+)@(?P<host>[^)]+)\) Password: ?$";

/// Passphrase prompt of `ssh` for a key file.
const SSH_PASSPHRASE: &str = r"^Enter passphrase for key '(?P<path>.+)': ?$";

/// Passphrase prompt of `ssh-add`, such as `Enter passphrase for (stdin): `
/// for a key read from stdin (DEC-015).
const SSH_ADD_PASSPHRASE: &str =
    r"^Enter passphrase for (?P<path>[^']+?)(?: \(will confirm each use\))?: ?$";

/// The key path that `ssh-add` shows for a key read from stdin; the askpass
/// entry of a [`SshAddKey`](crate::LaunchStep::SshAddKey) step uses it
/// (DEC-015).
pub(crate) const SSH_ADD_STDIN_KEY_PATH: &str = "(stdin)";

/// The compiled prompt patterns.
pub(super) struct PromptPatterns {
    password: [Regex; 2],
    passphrase: [Regex; 2],
}

impl PromptPatterns {
    /// Compiles the patterns.
    ///
    /// # Errors
    /// Only if a constant pattern is invalid, which the tests rule out.
    pub(super) fn new() -> Result<PromptPatterns, regex::Error> {
        Ok(PromptPatterns {
            password: [Regex::new(PASSWORD)?, Regex::new(KEYBOARD_INTERACTIVE)?],
            passphrase: [Regex::new(SSH_PASSPHRASE)?, Regex::new(SSH_ADD_PASSPHRASE)?],
        })
    }

    /// What `prompt` asks for, or `None` for a prompt askpass cannot answer
    /// (host key confirmation, one-time codes, …).
    pub(super) fn parse(&self, prompt: &str) -> Option<PromptMatcher> {
        let password = self
            .password
            .iter()
            .find_map(|pattern| pattern.captures(prompt));
        if let Some(captures) = password {
            return Some(PromptMatcher::Password {
                user: captures["user"].to_owned(),
                host: captures["host"].to_owned(),
            });
        }
        let passphrase = self
            .passphrase
            .iter()
            .find_map(|pattern| pattern.captures(prompt));
        passphrase.map(|captures| PromptMatcher::Passphrase {
            key_path: captures["path"].to_owned(),
        })
    }
}

/// Whether the entry `expected` answers the prompt `asked`. Host names are
/// compared without case, as DNS does.
pub(super) fn answers(expected: &PromptMatcher, asked: &PromptMatcher) -> bool {
    match (expected, asked) {
        (
            PromptMatcher::Password { user, host },
            PromptMatcher::Password {
                user: asked_user,
                host: asked_host,
            },
        ) => user == asked_user && host.eq_ignore_ascii_case(asked_host),
        (
            PromptMatcher::Passphrase { key_path },
            PromptMatcher::Passphrase {
                key_path: asked_path,
            },
        ) => key_path == asked_path,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn parse(prompt: &str) -> Option<PromptMatcher> {
        PromptPatterns::new().unwrap().parse(prompt)
    }

    fn password(user: &str, host: &str) -> Option<PromptMatcher> {
        Some(PromptMatcher::Password {
            user: user.into(),
            host: host.into(),
        })
    }

    fn passphrase(key_path: &str) -> Option<PromptMatcher> {
        Some(PromptMatcher::Passphrase {
            key_path: key_path.into(),
        })
    }

    #[test]
    fn ssh_password_prompt_is_recognized() {
        assert_eq!(
            parse("admin@web01.example.com's password: "),
            password("admin", "web01.example.com")
        );
    }

    #[test]
    fn keyboard_interactive_prompt_is_recognized() {
        assert_eq!(
            parse("(admin@192.0.2.10) Password: "),
            password("admin", "192.0.2.10")
        );
    }

    #[test]
    fn ssh_key_passphrase_prompt_is_recognized() {
        assert_eq!(
            parse("Enter passphrase for key '/home/user/.ssh/id_ed25519': "),
            passphrase("/home/user/.ssh/id_ed25519")
        );
    }

    #[test]
    fn ssh_add_stdin_passphrase_prompt_is_recognized() {
        assert_eq!(
            parse("Enter passphrase for (stdin): "),
            passphrase(SSH_ADD_STDIN_KEY_PATH)
        );
    }

    #[test]
    fn ssh_add_confirm_passphrase_prompt_is_recognized() {
        assert_eq!(
            parse("Enter passphrase for (stdin) (will confirm each use): "),
            passphrase(SSH_ADD_STDIN_KEY_PATH)
        );
    }

    #[test]
    fn host_key_confirmation_is_not_recognized() {
        let prompt = "Are you sure you want to continue connecting (yes/no/[fingerprint])? ";

        assert_eq!(parse(prompt), None);
    }

    #[test]
    fn host_comparison_ignores_case() {
        let expected = PromptMatcher::Password {
            user: "admin".into(),
            host: "Web01.Example.com".into(),
        };

        let matched = parse("admin@web01.example.com's password: ")
            .is_some_and(|asked| answers(&expected, &asked));

        assert!(matched);
    }

    #[test]
    fn different_user_does_not_match() {
        let expected = PromptMatcher::Password {
            user: "admin".into(),
            host: "web01.example.com".into(),
        };

        let matched = parse("root@web01.example.com's password: ")
            .is_some_and(|asked| answers(&expected, &asked));

        assert!(!matched);
    }
}
