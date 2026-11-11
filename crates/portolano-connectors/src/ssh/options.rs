//! The `-o Key=Value` options a document may give to `ssh` (DEC-026.1).
//!
//! A document may come from someone else, so it is not trusted like
//! `~/.ssh/config`. Instead of refusing the dangerous options, which would
//! always miss a keyword or an alias, only a closed list is accepted: options
//! that run no command, open no forwarding or tunnel, change neither the
//! destination, the user, the port, the agent, the environment nor the files
//! read (other than the key named by `IdentityFile`), and do not weaken the
//! check of the host key. Everything else goes in the user's `~/.ssh/config`.

/// The accepted options, compared without regard to case, each with the
/// reason it is safe.
const ALLOWED_OPTIONS: [&str; 23] = [
    // The key to log in with, and whether only the given keys are offered:
    // written by `add ssh --key` (DEC-022.2). Reading a key runs nothing.
    "IdentityFile",
    "IdentitiesOnly",
    // Timeouts and keepalives of the connection itself. `ServerAliveInterval`
    // is not here: it is the `keepalive_s` setting.
    "ConnectTimeout",
    "ConnectionAttempts",
    "ServerAliveCountMax",
    "TCPKeepAlive",
    // Which authentication methods to try, among those of the destination.
    "PreferredAuthentications",
    "PubkeyAuthentication",
    "PasswordAuthentication",
    "KbdInteractiveAuthentication",
    // Algorithms of the session: they run nothing, but they can enable weak
    // algorithms (such as `+3des-cbc`). An accepted risk: old servers need
    // them, and the choice is visible in the document.
    "Ciphers",
    "MACs",
    "KexAlgorithms",
    // Not `HostKeyAlgorithms`: an explicit list turns off the ordering
    // taken from `known_hosts`, so with `StrictHostKeyChecking=accept-new` a
    // man in the middle could have a key of a new type accepted and receive
    // the password through askpass.
    "PubkeyAcceptedAlgorithms",
    // Presentation and transport details.
    "Compression",
    "LogLevel",
    "RequestTTY",
    "AddressFamily",
    // Adds the key used to log in to the agent that is already running; it
    // changes neither which agent nor what is forwarded.
    "AddKeysToAgent",
    // Extra checks and displays of the host key: they only add safety.
    "CheckHostIP",
    "VisualHostKey",
    // Only with the values of `SAFE_HOST_KEY_CHECKING`.
    "StrictHostKeyChecking",
    // Hash new entries of the user's known hosts file: only their form.
    "HashKnownHosts",
];

/// Values of `StrictHostKeyChecking` that keep the host key checked; `no`
/// and `off` would accept any server.
const SAFE_HOST_KEY_CHECKING: [&str; 3] = ["yes", "ask", "accept-new"];

/// Checks one `-o Key=Value` option of a document.
///
/// The name must be a plain keyword (ssh reads `-o` like a configuration
/// line, so a space or a `=` would start another keyword) from
/// [`ALLOWED_OPTIONS`]; the value must hold no control character, which
/// could end the line.
///
/// # Errors
/// A description of the problem, in Italian.
pub(super) fn check_option(key: &str, value: &str) -> Result<(), String> {
    if key.is_empty() || !key.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(format!(
            "campo 'options': nome di opzione non valido '{key}'"
        ));
    }
    if value.chars().any(char::is_control) {
        return Err(format!(
            "campo 'options': il valore di '{key}' contiene un carattere di controllo"
        ));
    }
    let Some(allowed) = ALLOWED_OPTIONS
        .iter()
        .find(|name| name.eq_ignore_ascii_case(key))
    else {
        return Err(format!(
            "campo 'options': l'opzione '{key}' non è ammessa nel documento{}. Sono ammesse \
             solo opzioni che non eseguono comandi, non aprono inoltri, non cambiano \
             destinazione, utente, porta, agent, ambiente o file letti e non indeboliscono \
             la verifica della host key; se ti serve, mettila nel tuo ~/.ssh/config",
            alternative(key)
        ));
    };
    if *allowed == "StrictHostKeyChecking"
        && !SAFE_HOST_KEY_CHECKING
            .iter()
            .any(|safe| safe.eq_ignore_ascii_case(value.trim()))
    {
        return Err(format!(
            "campo 'options': '{key}' ammette solo {} nel documento, perché altri valori \
             accetterebbero qualsiasi server; se ti serve, mettilo nel tuo ~/.ssh/config",
            SAFE_HOST_KEY_CHECKING.join(", ")
        ));
    }
    Ok(())
}

/// The setting of the document that replaces a refused option, if any.
fn alternative(key: &str) -> &'static str {
    let replacements = [
        ("ProxyJump", " (per i jump host usa 'jump_hosts')"),
        (
            "ForwardAgent",
            " (per l'inoltro dell'agent usa 'agent_forwarding')",
        ),
        ("ServerAliveInterval", " (usa 'keepalive_s')"),
        ("ForwardX11", " (usa 'x11')"),
        ("ForwardX11Trusted", " (usa 'x11_trusted')"),
        ("Port", " (usa la porta della connessione)"),
        ("User", " (usa la credenziale della connessione)"),
        ("HostName", " (usa l'host della connessione)"),
        (
            "HostKeyAlgorithms",
            " (cambia quali chiavi dell'host vengono accettate e può indebolirne la verifica)",
        ),
    ];
    replacements
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(key))
        .map_or("", |(_, text)| text)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn every_listed_option_is_accepted_whatever_its_case() {
        for name in ALLOWED_OPTIONS {
            let value = if name == "StrictHostKeyChecking" {
                "yes"
            } else {
                "x"
            };
            assert_eq!(check_option(name, value), Ok(()), "{name}");
            assert_eq!(
                check_option(&name.to_ascii_lowercase(), value),
                Ok(()),
                "{name}"
            );
        }
    }

    #[test]
    fn options_that_run_programs_open_forwardings_or_change_the_login_are_refused() {
        let refused = [
            // DEC-024.1.
            "ProxyCommand",
            "ProxyJump",
            "LocalCommand",
            "PermitLocalCommand",
            "KnownHostsCommand",
            "PKCS11Provider",
            "SecurityKeyProvider",
            "ProxyUseFdpass",
            "SendEnv",
            "SetEnv",
            "Include",
            "Match",
            "IdentityAgent",
            "ControlPath",
            "ControlMaster",
            "RemoteCommand",
            "ForwardAgent",
            // DEC-026.1.
            "SmartcardDevice",
            "XAuthLocation",
            "HostName",
            "User",
            "Port",
            "HostKeyAlias",
            "RemoteForward",
            "DynamicForward",
            "LocalForward",
            "ForkAfterAuthentication",
            "UserKnownHostsFile",
            "GlobalKnownHostsFile",
            "ServerAliveInterval",
            "NumberOfPasswordPrompts",
            "Tunnel",
            "HostKeyAlgorithms",
        ];
        for name in refused {
            for key in [name.to_owned(), name.to_ascii_lowercase()] {
                let error = check_option(&key, "x").unwrap_err();

                assert!(error.contains(&format!("'{key}'")), "{error}");
                assert!(error.contains("~/.ssh/config"), "{error}");
            }
        }
    }

    #[test]
    fn host_key_checking_keeps_the_check() {
        for value in ["yes", "ask", "accept-new", "Accept-New"] {
            assert_eq!(
                check_option("StrictHostKeyChecking", value),
                Ok(()),
                "{value}"
            );
        }
        for value in ["no", "off", "false", ""] {
            let error = check_option("StrictHostKeyChecking", value).unwrap_err();

            assert!(error.contains("accept-new"), "{value}: {error}");
        }
    }

    #[test]
    fn host_key_algorithms_are_refused_and_left_to_the_ssh_config() {
        let error = check_option("HostKeyAlgorithms", "ssh-ed25519").unwrap_err();

        assert!(error.contains("'HostKeyAlgorithms'"), "{error}");
        assert!(error.contains("chiavi dell'host"), "{error}");
        assert!(error.contains("~/.ssh/config"), "{error}");
    }

    #[test]
    fn refused_options_point_at_the_setting_that_replaces_them() {
        assert!(
            check_option("ProxyJump", "x")
                .unwrap_err()
                .contains("jump_hosts")
        );
        assert!(
            check_option("ForwardAgent", "yes")
                .unwrap_err()
                .contains("agent_forwarding")
        );
        assert!(
            check_option("ServerAliveInterval", "30")
                .unwrap_err()
                .contains("keepalive_s")
        );
    }

    #[test]
    fn option_names_with_spaces_or_equals_are_refused() {
        for key in ["User ProxyCommand", "A=B", ""] {
            let error = check_option(key, "x").unwrap_err();

            assert!(error.contains("nome di opzione non valido"), "{error}");
        }
    }

    #[test]
    fn values_with_control_characters_are_refused() {
        let error = check_option("LogLevel", "INFO\nProxyCommand x").unwrap_err();

        assert!(error.contains("carattere di controllo"), "{error}");
    }
}
