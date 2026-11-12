//! Askpass client: asks the server of the session, or the user on the
//! terminal as a fallback (SPEC §7.2 points 3 and 5).

mod hidden_input;

use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::os::fd::AsFd;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

use portolano_core::Secret;

use self::hidden_input::HiddenInput;
use super::{ASKPASS_SOCK_ENV, ASKPASS_TOKEN_ENV, MAX_MESSAGE_BYTES, Reply, Request};
use crate::secret_input::{MAX_SECRET_INPUT_BYTES, SecretEnd, SecretInputError, read_secret};

/// How long the client waits for the answer of the server, which answers
/// immediately while it runs; after that the client asks on the terminal.
/// It only matters for a server that is stuck: a stopped one has closed its
/// socket, and the connection is refused at once.
pub(super) const REPLY_TIMEOUT: Duration = Duration::from_secs(5);

/// Prompts that need visible input: the host key confirmation of `ssh`.
const ECHO_MARKER: &str = "(yes/no";

/// Why the client could not produce an answer.
#[derive(Debug)]
pub(super) enum ClientError {
    /// A variable set by Portolano for `ssh` is missing.
    MissingVariable(&'static str),
    /// The server refused the request.
    Refused(String),
    /// Socket or terminal I/O failed.
    Io(io::Error),
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientError::MissingVariable(name) => write!(f, "variabile {name} assente"),
            ClientError::Refused(reason) => write!(f, "richiesta rifiutata dal server: {reason}"),
            ClientError::Io(error) => write!(f, "errore di I/O: {error}"),
        }
    }
}

impl From<io::Error> for ClientError {
    fn from(error: io::Error) -> Self {
        ClientError::Io(error)
    }
}

/// The outcome of an exchange with the server.
#[derive(Debug)]
pub(super) enum ServerAnswer {
    /// The answer to print.
    Answer(Secret),
    /// The server has nothing for this prompt: ask the user.
    Fallback,
}

/// Answers `prompt` on stdout, from the server or the terminal.
pub(super) fn run(prompt: &str) -> Result<(), ClientError> {
    let socket =
        std::env::var_os(ASKPASS_SOCK_ENV).ok_or(ClientError::MissingVariable(ASKPASS_SOCK_ENV))?;
    let token = std::env::var(ASKPASS_TOKEN_ENV)
        .map_err(|_| ClientError::MissingVariable(ASKPASS_TOKEN_ENV))?;
    let answer = match exchange(Path::new(&socket), &token, prompt, REPLY_TIMEOUT) {
        Ok(ServerAnswer::Answer(secret)) => secret,
        Ok(ServerAnswer::Fallback) => ask_on_tty(prompt)?,
        // A server that is gone or does not answer must not block the login:
        // the user can still type the password. A refusal (wrong token) is
        // not a failure of the server and stays an error.
        Err(ClientError::Io(error)) => {
            eprintln!(
                "portolano askpass: server non raggiungibile ({error}), chiedo sul terminale"
            );
            ask_on_tty(prompt)?
        }
        Err(error) => return Err(error),
    };
    // Straight to the descriptor: the buffer of `Stdout` would keep a copy
    // of the answer for the life of the process.
    let mut stdout = File::from(io::stdout().as_fd().try_clone_to_owned()?);
    stdout.write_all(answer.expose().as_bytes())?;
    stdout.write_all(b"\n")?;
    Ok(())
}

/// Sends one request to the server at `socket` and reads its reply,
/// waiting at most `timeout` for it.
///
/// # Errors
/// [`ClientError::Io`] for socket failures, a malformed reply or no reply
/// within `timeout`; [`ClientError::Refused`] if the server rejects the
/// request.
pub(super) fn exchange(
    socket: &Path,
    token: &str,
    prompt: &str,
    timeout: Duration,
) -> Result<ServerAnswer, ClientError> {
    let mut stream = UnixStream::connect(socket)?;
    stream.set_read_timeout(Some(timeout))?;
    let mut request = serde_json::to_vec(&Request { token, prompt }).map_err(io::Error::from)?;
    request.push(b'\n');
    stream.write_all(&request)?;

    // The reply holds the answer: it is read into a zeroed buffer of fixed
    // size (DEC-024.5). Without escapes `serde_json` copies the answer once,
    // into the `String` that becomes the `Secret`. Known limit: an answer
    // with characters that JSON escapes (quotes, backslashes, controls) is
    // first rebuilt in an internal buffer of `serde_json`, freed without
    // being zeroed.
    let line = read_secret(&mut stream, SecretEnd::Line, MAX_MESSAGE_BYTES).map_err(io_error)?;
    match serde_json::from_str::<Reply<String>>(line.expose()).map_err(io::Error::from)? {
        Reply::Answer(answer) => Ok(ServerAnswer::Answer(Secret::new(answer))),
        Reply::Fallback(_) => Ok(ServerAnswer::Fallback),
        Reply::Error(reason) => Err(ClientError::Refused(reason)),
    }
}

/// A failed read of the reply as an I/O error, so that the client falls
/// back to the terminal as for any other socket failure.
fn io_error(error: SecretInputError) -> io::Error {
    match error {
        SecretInputError::Io(error) => error,
        other => io::Error::new(io::ErrorKind::InvalidData, other.to_string()),
    }
}

/// Asks the user on `/dev/tty`: with echo for a host key confirmation,
/// hidden otherwise. Fails when there is no terminal.
fn ask_on_tty(prompt: &str) -> Result<Secret, ClientError> {
    let mut tty = OpenOptions::new().read(true).write(true).open("/dev/tty")?;
    if prompt.contains(ECHO_MARKER) {
        write_prompt(&mut tty, prompt)?;
        return read_line(&mut tty);
    }
    // The echo goes off before the prompt shows (DEC-028.2).
    let hidden = HiddenInput::start(&tty)?;
    write_prompt(&mut tty, prompt)?;
    let answer = read_line(&mut tty);
    drop(hidden);
    // The line end typed by the user was not echoed either.
    tty.write_all(b"\n")?;
    answer
}

fn write_prompt(tty: &mut File, prompt: &str) -> io::Result<()> {
    tty.write_all(prompt.as_bytes())?;
    tty.flush()
}

fn read_line(tty: &mut File) -> Result<Secret, ClientError> {
    read_secret(tty, SecretEnd::Line, MAX_SECRET_INPUT_BYTES)
        .map_err(|error| ClientError::Io(io_error(error)))
}
