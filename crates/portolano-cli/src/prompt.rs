//! Questions to the user on the terminal, and secrets read from stdin.
//!
//! Hidden input goes through `rpassword`, which reads `/dev/tty`. A prompt
//! is shown only when the user is at a terminal (stdin or stderr is one), so
//! scripts and tests never block on a question: they get an error that says
//! which option to use instead.

use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, IsTerminal, Write};

use portolano_connectors::secret_input::{SecretEnd, read_stdin_secret};
use portolano_core::vault::MASTER_PASSWORD_ENV;
use portolano_core::{MasterPasswordSource, Secret, VaultError};
use portolano_import::{ImportError, SecretField, SecretPrompter, SecretRequest};
use portolano_ops::connect::LoginRequest;
use portolano_ops::vault::{Login, MIN_MASTER_PASSWORD_CHARS};

use crate::failure::CliError;
use crate::output::visible;

/// Whether the user is at a terminal and can answer questions.
pub fn interactive() -> bool {
    io::stdin().is_terminal() || io::stderr().is_terminal()
}

/// A hidden answer, asked twice; empty answers are refused.
pub fn new_secret(label: &str) -> Result<Secret, CliError> {
    require_terminal(label)?;
    let first = hidden(&format!("{label}: "))?;
    if first.expose().is_empty() {
        return Err(CliError::EmptySecret);
    }
    let second = hidden(&format!("{label} (conferma): "))?;
    if first.expose() != second.expose() {
        return Err(CliError::SecretMismatch);
    }
    Ok(first)
}

/// One line of stdin, without the line ending; empty lines are refused, and
/// so are lines longer than [`MAX_SECRET_INPUT_BYTES`](portolano_connectors::secret_input::MAX_SECRET_INPUT_BYTES)
/// (DEC-024.5).
pub fn stdin_line() -> Result<Secret, CliError> {
    let secret = read_stdin_secret(SecretEnd::Line)?;
    if secret.expose().is_empty() {
        return Err(CliError::EmptySecret);
    }
    Ok(secret)
}

/// All of stdin (a private key); empty input is refused, and so is input
/// longer than [`MAX_SECRET_INPUT_BYTES`](portolano_connectors::secret_input::MAX_SECRET_INPUT_BYTES)
/// (DEC-024.5).
pub fn stdin_all() -> Result<Secret, CliError> {
    let secret = read_stdin_secret(SecretEnd::Eof)?;
    if secret.expose().trim().is_empty() {
        return Err(CliError::EmptySecret);
    }
    Ok(secret)
}

/// Asks a yes/no question on the terminal; the default is no.
pub fn confirm(question: &str) -> Result<bool, CliError> {
    require_terminal("la conferma")?;
    let answer = echoed(&format!("{question} [s/N] "))?;
    Ok(matches!(
        answer.trim(),
        "s" | "S" | "si" | "sì" | "y" | "yes"
    ))
}

/// Asks the login data for a credential "ask at launch"; `None` without a
/// terminal, so that the client asks instead.
pub fn login(request: &LoginRequest) -> Result<Option<Login>, CliError> {
    if !interactive() {
        return Ok(None);
    }
    let tty = Tty::open()?;
    eprintln!("Credenziali per {}", visible(&request.path));
    let username = tty.ask("Utente: ")?;
    let domain = if request.ask_domain {
        tty.ask("Dominio (invio per nessuno): ")?
    } else {
        String::new()
    };
    let password = hidden("Password: ")?;
    let some = |text: String| (!text.is_empty()).then_some(text);
    Ok(Some(Login {
        username: some(username),
        domain: some(domain),
        password: (!password.expose().is_empty()).then_some(password),
    }))
}

/// The master password from [`MASTER_PASSWORD_ENV`], or asked on the terminal
/// (twice when the vault is created).
pub struct MasterPassword;

impl MasterPasswordSource for MasterPassword {
    fn master_password(&self, creating: bool) -> Result<Secret, VaultError> {
        if let Some(value) = std::env::var_os(MASTER_PASSWORD_ENV).filter(|value| !value.is_empty())
        {
            return value
                .into_string()
                .map(Secret::new)
                .map_err(|_| unavailable(format!("{MASTER_PASSWORD_ENV} non è UTF-8 valido")));
        }
        let result = if creating {
            new_secret(&new_master_password_label())
        } else {
            let label = "Master password del vault";
            require_terminal(label).and_then(|()| hidden(&format!("{label}: ")))
        };
        result.map_err(|error| unavailable(error.to_string()))
    }
}

/// The prompt of a new master password, with the minimum length that the
/// vault requires (DEC-026.6).
fn new_master_password_label() -> String {
    format!("Nuova master password del vault (almeno {MIN_MASTER_PASSWORD_CHARS} caratteri)")
}

fn unavailable(reason: String) -> VaultError {
    VaultError::MasterPasswordUnavailable { reason }
}

/// Asks the encrypted secrets of an import, hidden; an empty answer skips.
pub struct ImportPrompter;

impl SecretPrompter for ImportPrompter {
    fn ask(&mut self, request: &SecretRequest) -> Result<Option<Secret>, ImportError> {
        let what = match &request.field {
            SecretField::Password => "password".to_owned(),
            SecretField::Passphrase => "passphrase".to_owned(),
            SecretField::PrivateKey => "chiave privata".to_owned(),
            SecretField::CustomProperty(name) => format!("proprietà '{name}'"),
        };
        let user = request
            .username
            .as_deref()
            .map(|user| format!(" (utente {user})"))
            .unwrap_or_default();
        let prompt = format!("{what} di {}{user}, invio per saltare: ", request.node_path);
        // The path, user and property name come from the imported file.
        let prompt = visible(&prompt);
        let secret = hidden(&prompt).map_err(|error| ImportError::Prompt {
            source: io::Error::other(error.to_string()),
        })?;
        Ok((!secret.expose().is_empty()).then_some(secret))
    }
}

fn require_terminal(what: &str) -> Result<(), CliError> {
    if interactive() {
        Ok(())
    } else {
        Err(CliError::NoTerminal {
            what: what.to_owned(),
        })
    }
}

fn hidden(prompt: &str) -> Result<Secret, CliError> {
    rpassword::prompt_password(prompt)
        .map(Secret::new)
        .map_err(CliError::Terminal)
}

/// An answer typed with echo, for questions that are not secrets.
fn echoed(prompt: &str) -> Result<String, CliError> {
    Tty::open()?.ask(prompt)
}

/// The controlling terminal, for questions with visible answers.
struct Tty {
    file: File,
}

impl Tty {
    fn open() -> Result<Tty, CliError> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/tty")
            .map_err(CliError::Terminal)?;
        Ok(Tty { file })
    }

    /// A terminal in canonical mode returns one line per read, so a reader
    /// per question loses nothing.
    fn ask(&self, prompt: &str) -> Result<String, CliError> {
        let mut output = &self.file;
        output
            .write_all(prompt.as_bytes())
            .and_then(|()| output.flush())
            .map_err(CliError::Terminal)?;
        let mut line = String::new();
        BufReader::new(&self.file)
            .read_line(&mut line)
            .map_err(CliError::Terminal)?;
        Ok(line.trim_end_matches(['\n', '\r']).to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_master_password_prompt_states_the_minimum_length() {
        assert!(new_master_password_label().contains("almeno 12 caratteri"));
    }
}
