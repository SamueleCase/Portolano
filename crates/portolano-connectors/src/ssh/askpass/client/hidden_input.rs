//! Hidden input on the terminal for the askpass fallback (DEC-028.2).
//!
//! The echo goes off *before* the prompt is written: a prompt shown while the
//! terminal still echoes lets what is typed or pasted right after it appear
//! in clear, as it did with `rpassword`.
//!
//! The echo must always come back, or the terminal of the user (or the tab
//! of the app) stays mute. Dropping [`HiddenInput`] restores it when the read
//! ends or fails. A signal that ends the process meanwhile (Ctrl-C, a hang
//! up) would skip the drop, so the guard handles them as `readpassphrase`
//! of OpenSSH does: the signals are blocked and a thread waits for them,
//! restores the terminal and raises the signal again, so that the process
//! still ends as the signal demands.

use std::io;
use std::os::fd::{AsFd, OwnedFd};

use nix::sys::signal::{SigSet, SigmaskHow, Signal, raise};
use nix::sys::termios::{LocalFlags, SetArg, Termios, tcgetattr, tcsetattr};

/// Signals whose default action ends the process and that a terminal or a
/// parent sends to interrupt a prompt.
const ENDING_SIGNALS: [Signal; 4] = [
    Signal::SIGHUP,
    Signal::SIGINT,
    Signal::SIGQUIT,
    Signal::SIGTERM,
];

/// Echo flags turned off while the input is hidden. The line stays
/// canonical, so the terminal still handles erasing and the line end.
const ECHO_FLAGS: LocalFlags = LocalFlags::ECHO
    .union(LocalFlags::ECHOE)
    .union(LocalFlags::ECHOK)
    .union(LocalFlags::ECHONL);

/// The terminal with its echo off; dropping it restores the terminal modes
/// and the signal mask of the thread.
pub(super) struct HiddenInput {
    saved: SavedModes,
    previous_mask: SigSet,
}

/// The modes of a terminal before the echo went off.
struct SavedModes {
    terminal: OwnedFd,
    original: Termios,
}

impl SavedModes {
    /// A copy with its own descriptor, for the signal thread: `Termios` is
    /// not `Sync`, so the two cannot share one.
    fn try_clone(&self) -> io::Result<SavedModes> {
        Ok(SavedModes {
            terminal: self.terminal.try_clone()?,
            original: self.original.clone(),
        })
    }

    /// Puts the original modes back. Best effort: there is nothing better to
    /// do with a terminal that refuses them.
    fn restore(&self) {
        let _ = tcsetattr(&self.terminal, SetArg::TCSANOW, &self.original);
    }
}

impl HiddenInput {
    /// Turns the echo of `terminal` off, after arranging for it to come back
    /// on drop and on the [`ENDING_SIGNALS`].
    ///
    /// # Errors
    /// The terminal modes cannot be read or set, or the signal thread cannot
    /// start; the terminal is then left as it was.
    pub(super) fn start(terminal: impl AsFd) -> io::Result<HiddenInput> {
        let original = tcgetattr(&terminal)?;
        let saved = SavedModes {
            terminal: terminal.as_fd().try_clone_to_owned()?,
            original,
        };
        let watched = saved.try_clone()?;
        let signals = ENDING_SIGNALS
            .into_iter()
            .fold(SigSet::empty(), |mut set, signal| {
                set.add(signal);
                set
            });
        // Blocked before the thread starts, which inherits the mask: blocked
        // in every thread, the signals wait for `sigwait` instead of ending
        // the process.
        let previous_mask = signals.thread_swap_mask(SigmaskHow::SIG_BLOCK)?;
        let guard = HiddenInput {
            saved,
            previous_mask,
        };
        std::thread::Builder::new()
            .name("portolano-askpass-signals".to_owned())
            .spawn(move || restore_on_signal(&signals, &watched))?;

        let mut hidden = guard.saved.original.clone();
        hidden.local_flags.remove(ECHO_FLAGS);
        tcsetattr(&guard.saved.terminal, SetArg::TCSANOW, &hidden)?;
        Ok(guard)
    }
}

impl Drop for HiddenInput {
    fn drop(&mut self) {
        self.saved.restore();
        // A signal that arrived meanwhile is delivered now, with its default
        // action, to a terminal already restored.
        let _ = self.previous_mask.thread_set_mask();
    }
}

/// Waits for one of `signals`, restores the terminal and raises the signal
/// again with the signal unblocked in this thread, so that its default action
/// ends the process.
fn restore_on_signal(signals: &SigSet, terminal: &SavedModes) {
    let Ok(signal) = signals.wait() else {
        return;
    };
    terminal.restore();
    if SigSet::from(signal).thread_unblock().is_ok() {
        let _ = raise(signal);
    }
}

#[cfg(test)]
mod tests {
    use nix::pty::{OpenptyResult, openpty};
    use pretty_assertions::assert_eq;

    use super::*;

    fn terminal() -> OpenptyResult {
        openpty(None, None).unwrap()
    }

    fn local_flags(terminal: &OpenptyResult) -> LocalFlags {
        tcgetattr(&terminal.slave).unwrap().local_flags
    }

    #[test]
    fn echo_is_off_while_the_input_is_hidden() {
        let terminal = terminal();
        assert!(local_flags(&terminal).contains(LocalFlags::ECHO));

        let _hidden = HiddenInput::start(&terminal.slave).unwrap();

        assert!(!local_flags(&terminal).intersects(ECHO_FLAGS));
        assert!(local_flags(&terminal).contains(LocalFlags::ICANON));
    }

    #[test]
    fn dropping_the_guard_restores_the_original_modes() {
        let terminal = terminal();
        let original = tcgetattr(&terminal.slave).unwrap();

        drop(HiddenInput::start(&terminal.slave).unwrap());

        assert_eq!(tcgetattr(&terminal.slave).unwrap(), original);
    }

    #[test]
    fn dropping_the_guard_restores_the_signal_mask() {
        let terminal = terminal();
        let before = SigSet::thread_get_mask().unwrap();

        let hidden = HiddenInput::start(&terminal.slave).unwrap();
        assert!(SigSet::thread_get_mask().unwrap().contains(Signal::SIGINT));
        drop(hidden);

        assert_eq!(
            SigSet::thread_get_mask().unwrap().contains(Signal::SIGINT),
            before.contains(Signal::SIGINT)
        );
    }
}
