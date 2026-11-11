//! Askpass server: answers prompts over a Unix socket in a private
//! temporary directory, each secret at most once (SPEC §7.2 points 1, 4, 6).

use std::fs;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

use portolano_core::Secret;
use tempfile::TempDir;
use zeroize::Zeroizing;

use super::prompt::{PromptPatterns, answers};
use super::{
    ASKPASS_ENABLED, ASKPASS_ENV, ASKPASS_SOCK_ENV, ASKPASS_TOKEN_ENV, MAX_MESSAGE_BYTES, Reply,
    Request,
};
use crate::ConnectError;
use crate::plan::PromptMatcher;

/// How long the server waits for a client to send its request, so that a
/// stuck client cannot block the next prompt.
const READ_TIMEOUT: Duration = Duration::from_secs(10);

/// A secret the server may answer with, already read from the vault.
pub(crate) struct ServedSecret {
    /// The prompt it answers.
    pub(crate) matcher: PromptMatcher,
    /// The answer.
    pub(crate) secret: Secret,
}

/// A running askpass server. Dropping it stops the thread, removes the
/// socket and its directory, and zeroes the secrets not yet served.
pub(crate) struct AskpassServer {
    socket: PathBuf,
    token: String,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
    // Declared last: dropped after `drop` has joined the thread, so the
    // directory disappears only once nobody uses the socket.
    _dir: TempDir,
}

impl AskpassServer {
    /// Creates the private directory and the socket and starts serving
    /// `secrets`.
    ///
    /// # Errors
    /// [`ConnectError::Io`] if the directory, the socket or the thread cannot
    /// be created; [`ConnectError::Askpass`] if no random token is available.
    pub(crate) fn start(secrets: Vec<ServedSecret>) -> Result<AskpassServer, ConnectError> {
        let dir = tempfile::Builder::new()
            .prefix("portolano-askpass-")
            .tempdir()
            .map_err(|source| io_error("creazione della directory dell'askpass", source))?;
        fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o700))
            .map_err(|source| io_error("permessi della directory dell'askpass", source))?;
        let socket = dir.path().join("askpass.sock");
        let listener = UnixListener::bind(&socket)
            .map_err(|source| io_error("creazione del socket dell'askpass", source))?;
        let token = random_token()?;
        let patterns = PromptPatterns::new().map_err(|error| ConnectError::Askpass {
            reason: format!("espressione dei prompt non valida: {error}"),
        })?;

        let state = ServerState {
            token: token.clone(),
            patterns,
            secrets: secrets.into_iter().map(Some).collect(),
        };
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        let thread = std::thread::Builder::new()
            .name("portolano-askpass".to_owned())
            .spawn(move || serve(&listener, state, &thread_stop))
            .map_err(|source| io_error("avvio del thread dell'askpass", source))?;

        Ok(AskpassServer {
            socket,
            token,
            stop,
            thread: Some(thread),
            _dir: dir,
        })
    }

    /// The path of the socket; the code uses [`AskpassServer::child_env`].
    #[cfg(test)]
    pub(crate) fn socket(&self) -> &Path {
        &self.socket
    }

    /// The one-time token of the session.
    #[cfg(test)]
    pub(crate) fn token(&self) -> &str {
        &self.token
    }

    /// The environment that makes a child (`ssh`, `ssh-add`) use this server
    /// through `self_exe` as its askpass program (SPEC §7.2 point 2).
    pub(crate) fn child_env(&self, self_exe: &Path) -> Vec<(String, String)> {
        vec![
            (
                "SSH_ASKPASS".to_owned(),
                self_exe.to_string_lossy().into_owned(),
            ),
            ("SSH_ASKPASS_REQUIRE".to_owned(), "force".to_owned()),
            (ASKPASS_ENV.to_owned(), ASKPASS_ENABLED.to_owned()),
            (
                ASKPASS_SOCK_ENV.to_owned(),
                self.socket.to_string_lossy().into_owned(),
            ),
            (ASKPASS_TOKEN_ENV.to_owned(), self.token.clone()),
        ]
    }
}

impl Drop for AskpassServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        // The thread blocks in `accept`: a connection wakes it up so that it
        // sees the flag. Failures are ignored because there is nothing left
        // to do in a destructor; the directory is removed anyway.
        let _ = UnixStream::connect(&self.socket);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// The state owned by the server thread; dropping it zeroes the secrets.
struct ServerState {
    token: String,
    patterns: PromptPatterns,
    /// `None` once served: each secret is answered at most once, so a wrong
    /// password does not loop (SPEC §7.2 point 4).
    secrets: Vec<Option<ServedSecret>>,
}

fn serve(listener: &UnixListener, mut state: ServerState, stop: &AtomicBool) {
    for stream in listener.incoming() {
        if stop.load(Ordering::SeqCst) {
            break;
        }
        let Ok(stream) = stream else {
            continue;
        };
        if let Err(error) = state.handle(&stream) {
            tracing::debug!(%error, "askpass: richiesta non gestita");
        }
    }
}

impl ServerState {
    fn handle(&mut self, stream: &UnixStream) -> io::Result<()> {
        stream.set_read_timeout(Some(READ_TIMEOUT))?;
        let mut line = String::new();
        BufReader::new(stream.take(MAX_MESSAGE_BYTES)).read_line(&mut line)?;
        let request: Request<String> = match serde_json::from_str(&line) {
            Ok(request) => request,
            Err(_) => return write_reply(stream, &Reply::Error("richiesta non valida")),
        };
        if !constant_time_eq(request.token.as_bytes(), self.token.as_bytes()) {
            tracing::warn!("askpass: richiesta con token non valido rifiutata");
            return write_reply(stream, &Reply::Error("token non valido"));
        }
        match self.take_answer(&request.prompt) {
            Some(served) => write_reply(stream, &Reply::Answer(served.secret.expose())),
            None => write_reply(stream, &Reply::<&str>::Fallback(true)),
        }
    }

    /// Removes and returns the first unserved secret that answers `prompt`.
    fn take_answer(&mut self, prompt: &str) -> Option<ServedSecret> {
        let asked = self.patterns.parse(prompt)?;
        self.secrets
            .iter_mut()
            .find(|slot| {
                slot.as_ref()
                    .is_some_and(|served| answers(&served.matcher, &asked))
            })
            .and_then(Option::take)
    }
}

fn write_reply(mut stream: &UnixStream, reply: &Reply<&str>) -> io::Result<()> {
    let size = match reply {
        Reply::Answer(text) | Reply::Error(text) => text.len(),
        Reply::Fallback(_) => 0,
    };
    // Room for the worst-case JSON escaping (`\u00XX`): the buffer never
    // reallocates, so no unzeroed copy of an answer is left on the heap.
    let mut buffer = Zeroizing::new(Vec::with_capacity(size * 6 + 32));
    serde_json::to_writer(&mut *buffer, reply)?;
    buffer.push(b'\n');
    stream.write_all(&buffer)?;
    stream.flush()
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .fold(0u8, |difference, (a, b)| difference | (a ^ b))
            == 0
}

fn random_token() -> Result<String, ConnectError> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).map_err(|error| ConnectError::Askpass {
        reason: format!("generatore casuale non disponibile: {error}"),
    })?;
    Ok(hex::encode(bytes))
}

fn io_error(context: &str, source: io::Error) -> ConnectError {
    ConnectError::Io {
        context: context.to_owned(),
        source,
    }
}
