//! Where a session sends what the tab shows (SPEC-UI §4.4): output frames as
//! raw bytes and states as JSON. The app connects them to Tauri channels;
//! the tests to plain queues.

use super::model::SessionState;

/// Version of the frame header; a frontend that reads another version
/// ignores the frame.
pub const FRAME_VERSION: u8 = 1;

/// What the payload of a frame holds. The integrated RDP panel will add its
/// own kinds (bitmaps, pointer) without a new channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FrameKind {
    /// Bytes written by the program of a terminal session, for xterm.js.
    TerminalOutput = 0,
}

/// An output frame: the header `[version, kind]` then the payload.
pub fn frame(kind: FrameKind, payload: &[u8]) -> Vec<u8> {
    let mut frame = Vec::with_capacity(payload.len() + 2);
    frame.push(FRAME_VERSION);
    frame.push(kind as u8);
    frame.extend_from_slice(payload);
    frame
}

/// The receiving end of a session. Delivery is best effort: a frontend that
/// is gone (tab closed, window reloaded) must not stop the session, whose
/// output is still read so that the program never blocks on it.
pub trait SessionSink: Send + Sync + 'static {
    /// An output frame (see [`frame`]).
    fn output(&self, frame: Vec<u8>);

    /// A new state.
    fn state(&self, state: &SessionState);
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn frame_starts_with_version_and_kind() {
        assert_eq!(
            frame(FrameKind::TerminalOutput, b"ok"),
            vec![FRAME_VERSION, 0, b'o', b'k']
        );
    }
}
