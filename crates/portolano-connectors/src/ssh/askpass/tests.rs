//! Integration tests of the askpass protocol: the real server and the real
//! client exchange over the socket (SPEC §7.4).

use std::path::Path;

use portolano_core::Secret;
use pretty_assertions::assert_eq;

use super::client::{ClientError, ServerAnswer, exchange};
use super::*;
use crate::plan::PromptMatcher;

const PASSWORD: &str = "sentinel-askpass-password";
const PASSPHRASE: &str = "sentinel-askpass-passphrase";

fn password_entry(user: &str, host: &str, value: &str) -> ServedSecret {
    ServedSecret {
        matcher: PromptMatcher::Password {
            user: user.into(),
            host: host.into(),
        },
        secret: Secret::new(value.into()),
    }
}

fn passphrase_entry(key_path: &str, value: &str) -> ServedSecret {
    ServedSecret {
        matcher: PromptMatcher::Passphrase {
            key_path: key_path.into(),
        },
        secret: Secret::new(value.into()),
    }
}

fn ask(server: &AskpassServer, prompt: &str) -> Option<String> {
    match exchange(server.socket(), server.token(), prompt).unwrap() {
        ServerAnswer::Answer(secret) => Some(secret.expose().to_owned()),
        ServerAnswer::Fallback => None,
    }
}

fn start_default() -> AskpassServer {
    AskpassServer::start(vec![
        password_entry("admin", "web01.example.com", PASSWORD),
        passphrase_entry("/home/user/.ssh/id_ed25519", PASSPHRASE),
        passphrase_entry(SSH_ADD_STDIN_KEY_PATH, PASSPHRASE),
    ])
    .unwrap()
}

#[test]
fn ssh_password_prompt_gets_the_password() {
    let server = start_default();

    let answer = ask(&server, "admin@web01.example.com's password: ");

    assert_eq!(answer.as_deref(), Some(PASSWORD));
}

#[test]
fn keyboard_interactive_prompt_gets_the_password() {
    let server = start_default();

    let answer = ask(&server, "(admin@web01.example.com) Password: ");

    assert_eq!(answer.as_deref(), Some(PASSWORD));
}

#[test]
fn ssh_key_passphrase_prompt_gets_the_passphrase() {
    let server = start_default();

    let answer = ask(
        &server,
        "Enter passphrase for key '/home/user/.ssh/id_ed25519': ",
    );

    assert_eq!(answer.as_deref(), Some(PASSPHRASE));
}

#[test]
fn ssh_add_passphrase_prompt_gets_the_passphrase() {
    let server = start_default();

    let answer = ask(&server, "Enter passphrase for (stdin): ");

    assert_eq!(answer.as_deref(), Some(PASSPHRASE));
}

#[test]
fn unknown_prompt_gets_a_fallback() {
    let server = start_default();

    let answer = ask(&server, "Verification code: ");

    assert_eq!(answer, None);
}

#[test]
fn second_request_for_the_same_prompt_gets_a_fallback() {
    let server = start_default();
    let prompt = "admin@web01.example.com's password: ";
    let first = ask(&server, prompt);

    let second = ask(&server, prompt);

    assert_eq!(first.as_deref(), Some(PASSWORD));
    assert_eq!(second, None);
}

#[test]
fn two_entries_for_the_same_prompt_are_served_in_order() {
    let server = AskpassServer::start(vec![
        password_entry("admin", "web01.example.com", "first-sentinel"),
        password_entry("admin", "web01.example.com", "second-sentinel"),
    ])
    .unwrap();
    let prompt = "admin@web01.example.com's password: ";

    let answers = [ask(&server, prompt), ask(&server, prompt)];

    assert_eq!(
        answers,
        [
            Some("first-sentinel".to_owned()),
            Some("second-sentinel".to_owned())
        ]
    );
}

#[test]
fn missing_server_is_an_io_error_so_the_client_asks_on_the_terminal() {
    let dir = tempfile::tempdir().unwrap();

    let result = exchange(&dir.path().join("askpass.sock"), "token", "Password: ");

    assert!(matches!(result, Err(ClientError::Io(_))), "{result:?}");
}

#[test]
fn silent_server_is_an_io_error_after_the_timeout() {
    let dir = tempfile::tempdir().unwrap();
    let socket = dir.path().join("askpass.sock");
    // Bound but never accepting, like a server that gave up.
    let _listener = std::os::unix::net::UnixListener::bind(&socket).unwrap();

    let result = exchange(&socket, "token", "Password: ");

    assert!(matches!(result, Err(ClientError::Io(_))), "{result:?}");
}

#[test]
fn wrong_token_is_refused() {
    let server = start_default();
    let wrong = "0".repeat(64);

    let result = exchange(
        server.socket(),
        &wrong,
        "admin@web01.example.com's password: ",
    );

    assert!(matches!(result, Err(ClientError::Refused(_))), "{result:?}");
}

#[test]
fn refused_request_does_not_consume_the_secret() {
    let server = start_default();
    let prompt = "admin@web01.example.com's password: ";
    let _ = exchange(server.socket(), "wrong-token", prompt);

    let answer = ask(&server, prompt);

    assert_eq!(answer.as_deref(), Some(PASSWORD));
}

#[test]
fn token_is_64_hex_digits() {
    let server = start_default();

    let token = server.token();

    assert_eq!(token.len(), 64);
    assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn socket_directory_is_private() {
    use std::os::unix::fs::PermissionsExt;
    let server = start_default();

    let mode = server
        .socket()
        .parent()
        .unwrap()
        .metadata()
        .unwrap()
        .permissions()
        .mode();

    assert_eq!(mode & 0o777, 0o700);
}

#[test]
fn drop_removes_the_socket_and_its_directory() {
    let server = start_default();
    let dir = server.socket().parent().unwrap().to_path_buf();

    drop(server);

    assert!(!dir.exists());
}

#[test]
fn child_env_points_ssh_to_the_server_without_secrets() {
    let server = start_default();

    let env = server.child_env(Path::new("/usr/local/bin/portolano"));

    let get = |name: &str| {
        env.iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    };
    assert_eq!(get("SSH_ASKPASS"), Some("/usr/local/bin/portolano"));
    assert_eq!(get("SSH_ASKPASS_REQUIRE"), Some("force"));
    assert_eq!(get(ASKPASS_ENV), Some(ASKPASS_ENABLED));
    assert_eq!(get(ASKPASS_SOCK_ENV), server.socket().to_str());
    assert_eq!(get(ASKPASS_TOKEN_ENV), Some(server.token()));
    assert!(
        env.iter()
            .all(|(_, value)| !value.contains(PASSWORD) && !value.contains(PASSPHRASE))
    );
}

#[test]
fn password_prompt_for_another_host_gets_a_fallback() {
    let server = start_default();

    let answer = ask(&server, "admin@evil.example.com's password: ");

    assert_eq!(answer, None);
}

#[test]
fn keyboard_interactive_prompt_for_another_user_gets_a_fallback() {
    let server = start_default();

    let answer = ask(&server, "(root@web01.example.com) Password: ");

    assert_eq!(answer, None);
}

#[test]
fn drop_returns_promptly_even_if_the_socket_has_disappeared() {
    let server = start_default();
    std::fs::remove_file(server.socket()).unwrap();
    let started = std::time::Instant::now();

    drop(server);

    assert!(started.elapsed() < std::time::Duration::from_secs(5));
}
