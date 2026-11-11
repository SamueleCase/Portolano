//! Reading secrets from pipes, sockets and terminals (DEC-024.5), shared by
//! the askpass client and the CLI (`--password-stdin`, `--key-stdin`).
//!
//! A secret is read into a single buffer, allocated once at its maximum size
//! and zeroed when dropped: a buffer that grows while reading would leave
//! copies of the secret in freed memory. Bytes go straight from the reader
//! into that buffer, without the internal buffer of `Stdin` or of a
//! `BufReader`, which would keep a copy. Input beyond the limit is refused.

use std::io::{self, Read};

use portolano_core::Secret;
use zeroize::{Zeroize, Zeroizing};

/// Largest secret read from stdin: 64 KiB hold the longest private keys in
/// PEM (an RSA key of 16384 bits takes about 13 KiB) with ample margin,
/// while a larger input is a mistake, such as the wrong file piped, that
/// should not end up in the vault.
pub const MAX_SECRET_INPUT_BYTES: usize = 64 * 1024;

/// Where a secret ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretEnd {
    /// At the first line feed, or at the end of the input. The line ending
    /// (`\n` or `\r\n`) is not part of the secret, and nothing after it is
    /// read.
    Line,
    /// At the end of the input; everything read is the secret.
    Eof,
}

/// Why a secret could not be read. The messages never contain the input.
#[derive(Debug, thiserror::Error)]
pub enum SecretInputError {
    /// The input is longer than the limit.
    #[error("il segreto supera il limite di {limit} byte")]
    TooLarge {
        /// The limit, in bytes.
        limit: usize,
    },
    /// The input is not UTF-8 text.
    #[error("il segreto non è testo UTF-8 valido")]
    NotUtf8,
    /// Reading failed.
    #[error("lettura del segreto non riuscita")]
    Io(#[source] io::Error),
}

/// Reads a secret of at most `limit` bytes from `reader`, up to `end`.
///
/// With [`SecretEnd::Line`] the reader is read one byte at a time, so that
/// nothing past the line is consumed.
///
/// # Errors
/// [`SecretInputError::TooLarge`] beyond `limit` bytes (line ending
/// excluded), [`SecretInputError::NotUtf8`], [`SecretInputError::Io`].
pub fn read_secret(
    reader: &mut impl Read,
    end: SecretEnd,
    limit: usize,
) -> Result<Secret, SecretInputError> {
    // One byte of room past the limit tells a secret of exactly `limit`
    // bytes from a longer one.
    let mut buffer = Zeroizing::new(vec![0_u8; limit.saturating_add(1)]);
    let mut filled = 0;
    loop {
        if filled == buffer.len() {
            return Err(SecretInputError::TooLarge { limit });
        }
        let window = match end {
            SecretEnd::Line => filled..filled + 1,
            SecretEnd::Eof => filled..buffer.len(),
        };
        let read = match reader.read(&mut buffer[window]) {
            Ok(0) => break,
            Ok(read) => read,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(SecretInputError::Io(error)),
        };
        filled += read;
        if end == SecretEnd::Line && buffer[filled - 1] == b'\n' {
            filled -= 1;
            if filled > 0 && buffer[filled - 1] == b'\r' {
                filled -= 1;
            }
            break;
        }
    }
    if filled > limit {
        return Err(SecretInputError::TooLarge { limit });
    }
    buffer.truncate(filled);
    // The allocation moves into the string, which `Secret` zeroes in full
    // (spare capacity included) when dropped.
    match String::from_utf8(std::mem::take(&mut *buffer)) {
        Ok(text) => Ok(Secret::new(text)),
        Err(error) => {
            error.into_bytes().zeroize();
            Err(SecretInputError::NotUtf8)
        }
    }
}

/// Reads a secret of at most [`MAX_SECRET_INPUT_BYTES`] from the standard
/// input of the process, as [`read_secret`] does.
///
/// # Errors
/// As [`read_secret`].
pub fn read_stdin_secret(end: SecretEnd) -> Result<Secret, SecretInputError> {
    read_secret(&mut unbuffered_stdin()?, end, MAX_SECRET_INPUT_BYTES)
}

/// The standard input without the buffer of `Stdin`, which would keep a
/// copy of what it reads for the life of the process.
#[cfg(unix)]
fn unbuffered_stdin() -> Result<std::fs::File, SecretInputError> {
    use std::os::fd::AsFd;

    let descriptor = io::stdin()
        .as_fd()
        .try_clone_to_owned()
        .map_err(SecretInputError::Io)?;
    Ok(std::fs::File::from(descriptor))
}

#[cfg(not(unix))]
fn unbuffered_stdin() -> Result<io::Stdin, SecretInputError> {
    Ok(io::stdin())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use pretty_assertions::assert_eq;

    use super::*;

    fn read(input: &str, end: SecretEnd, limit: usize) -> Result<String, SecretInputError> {
        read_secret(&mut Cursor::new(input.as_bytes()), end, limit)
            .map(|secret| secret.expose().to_owned())
    }

    #[test]
    fn line_stops_at_the_line_feed_and_drops_the_line_ending() {
        assert_eq!(
            read("sentinel\nrest", SecretEnd::Line, 64).unwrap(),
            "sentinel"
        );
        assert_eq!(
            read("sentinel\r\n", SecretEnd::Line, 64).unwrap(),
            "sentinel"
        );
        assert_eq!(read("sentinel", SecretEnd::Line, 64).unwrap(), "sentinel");
        assert_eq!(read("\n", SecretEnd::Line, 64).unwrap(), "");
    }

    #[test]
    fn line_does_not_consume_what_follows() {
        let mut input = Cursor::new(b"first\nsecond\n".to_vec());

        read_secret(&mut input, SecretEnd::Line, 64).unwrap();

        assert_eq!(input.position(), 6);
    }

    #[test]
    fn eof_keeps_everything() {
        let key = "-----BEGIN KEY-----\nAAAA\n-----END KEY-----\n";

        assert_eq!(read(key, SecretEnd::Eof, 64).unwrap(), key);
    }

    #[test]
    fn a_secret_of_exactly_the_limit_is_accepted() {
        let secret = "x".repeat(16);

        assert_eq!(read(&secret, SecretEnd::Eof, 16).unwrap(), secret);
        assert_eq!(
            read(&format!("{secret}\n"), SecretEnd::Line, 16).unwrap(),
            secret
        );
    }

    #[test]
    fn a_secret_beyond_the_limit_is_refused() {
        let secret = "x".repeat(17);

        for end in [SecretEnd::Line, SecretEnd::Eof] {
            let error = read(&secret, end, 16).unwrap_err();

            assert!(
                matches!(error, SecretInputError::TooLarge { limit: 16 }),
                "{error}"
            );
            assert!(!error.to_string().contains(&secret));
        }
    }

    #[test]
    fn invalid_utf8_is_refused() {
        let error = read_secret(&mut Cursor::new(vec![0xff, 0xfe]), SecretEnd::Eof, 16)
            .map(drop)
            .unwrap_err();

        assert!(matches!(error, SecretInputError::NotUtf8), "{error}");
    }

    /// A reader that fails once with `Interrupted`, as a signal would.
    struct Interrupting {
        input: Cursor<&'static [u8]>,
        interrupted: bool,
    }

    impl Read for Interrupting {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            if !self.interrupted {
                self.interrupted = true;
                return Err(io::ErrorKind::Interrupted.into());
            }
            self.input.read(buf)
        }
    }

    #[test]
    fn interrupted_reads_are_retried() {
        let mut reader = Interrupting {
            input: Cursor::new(b"sentinel\n"),
            interrupted: false,
        };

        let secret = read_secret(&mut reader, SecretEnd::Line, 64).unwrap();

        assert_eq!(secret.expose(), "sentinel");
    }
}
