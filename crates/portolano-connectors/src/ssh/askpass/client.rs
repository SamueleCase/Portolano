//! Askpass client: asks the server of the session, or the user on the
//! terminal as a fallback (SPEC §7.2 points 3 and 5).

use std::fmt;
use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

use portolano_core::Secret;
use zeroize::Zeroizing;

use super::{ASKPASS_SOCK_ENV, ASKPASS_TOKEN_ENV, MAX_MESSAGE_BYTES, Reply, Request};

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
    let mut stdout = io::stdout().lock();
    stdout.write_all(answer.expose().as_bytes())?;
    stdout.write_all(b"\n")?;
    stdout.flush()?;
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

    // Preallocated so that reading a typical answer does not reallocate and
    // leave unzeroed copies behind.
    let mut line = Zeroizing::new(String::with_capacity(4096));
    BufReader::new(stream.take(MAX_MESSAGE_BYTES)).read_line(&mut line)?;
    match serde_json::from_str::<Reply<String>>(&line).map_err(io::Error::from)? {
        Reply::Answer(answer) => Ok(ServerAnswer::Answer(Secret::new(answer))),
        Reply::Fallback(_) => Ok(ServerAnswer::Fallback),
        Reply::Error(reason) => Err(ClientError::Refused(reason)),
    }
}

/// Asks the user on `/dev/tty`: with echo for a host key confirmation,
/// hidden otherwise. Fails when there is no terminal.
fn ask_on_tty(prompt: &str) -> Result<Secret, ClientError> {
    if !prompt.contains(ECHO_MARKER) {
        return Ok(Secret::new(rpassword::prompt_password(prompt)?));
    }
    let mut tty = OpenOptions::new().read(true).write(true).open("/dev/tty")?;
    tty.write_all(prompt.as_bytes())?;
    tty.flush()?;
    let mut line = String::new();
    BufReader::new(tty).read_line(&mut line)?;
    let answer = line.trim_end_matches(['\r', '\n']).to_owned();
    Ok(Secret::new(answer))
}
