//! What the session commands exchange with the frontend: their inputs, the
//! channels of a session and their errors.

use portolano_connectors::RdpClient;
use portolano_ops::OpsError;
use portolano_ops::vault::Login;
use serde::Deserialize;
use specta::datatype::DataType;
use tauri::ipc::{Channel, InvokeResponseBody, IpcResponse};

use crate::dto::SecretText;
use crate::error::{ErrorCode, IpcError};
use crate::sessions::{SessionError, SessionSink, SessionState};

/// The options of a connection chosen in the interface.
#[derive(Debug, Clone, Default, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct SessionOptionsInput {
    /// The RDP client, over the connection and the configuration
    /// (DEC-016.7); none follows them.
    pub rdp_client: Option<RdpClientInput>,
}

/// An RDP client, as the interface names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, specta::Type)]
#[serde(rename_all = "kebab-case")]
pub enum RdpClientInput {
    /// FreeRDP.
    Freerdp,
    /// Windows App.
    WindowsApp,
}

impl From<RdpClientInput> for RdpClient {
    fn from(client: RdpClientInput) -> Self {
        match client {
            RdpClientInput::Freerdp => RdpClient::FreeRdp,
            RdpClientInput::WindowsApp => RdpClient::WindowsApp,
        }
    }
}

/// The login typed for a credential "ask at launch" (DEC-016.3). The
/// password becomes a secret as the request is read (SPEC-UI §1.2) and
/// lives in a vault of this session only.
#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct LoginInput {
    /// User name.
    pub username: Option<String>,
    /// Windows domain.
    pub domain: Option<String>,
    /// Password.
    pub password: Option<SecretText>,
}

impl From<LoginInput> for Login {
    fn from(login: LoginInput) -> Self {
        Login {
            username: login.username,
            domain: login.domain,
            password: login.password.map(SecretText::into_secret),
        }
    }
}

/// An output frame of a session (see [`crate::sessions::frame`]), sent as
/// raw bytes: the frontend receives an `ArrayBuffer`, not a JSON array of
/// numbers.
pub struct OutputFrame(pub Vec<u8>);

impl IpcResponse for OutputFrame {
    fn body(self) -> tauri::Result<InvokeResponseBody> {
        Ok(InvokeResponseBody::Raw(self.0))
    }
}

impl specta::Type for OutputFrame {
    fn definition(_: &mut specta::Types) -> DataType {
        DataType::Reference(specta_typescript::define("ArrayBuffer"))
    }
}

/// A session reporting to the two channels the frontend passed to
/// `open_session`. A channel whose window is gone is not an error: the
/// session goes on until it is closed.
pub struct ChannelSink {
    /// The output frames.
    pub output: Channel<OutputFrame>,
    /// The states.
    pub states: Channel<SessionState>,
}

impl SessionSink for ChannelSink {
    fn output(&self, frame: Vec<u8>) {
        let _ = self.output.send(OutputFrame(frame));
    }

    fn state(&self, state: &SessionState) {
        let _ = self.states.send(state.clone());
    }
}

impl From<SessionError> for IpcError {
    fn from(error: SessionError) -> Self {
        // The connectors know whether the vault is locked, a client missing,
        // …: their errors keep their own codes.
        if let SessionError::Connect(connect) = error {
            return IpcError::from(OpsError::from(connect));
        }
        let code = match &error {
            SessionError::NotFound { .. } => ErrorCode::NotFound,
            SessionError::Unsupported { .. } => ErrorCode::Invalid,
            SessionError::Io { .. } => ErrorCode::Io,
            SessionError::Connect(_)
            | SessionError::StdinSecretInTerminal
            | SessionError::NotASession { .. }
            | SessionError::Terminal { .. }
            | SessionError::Spawn { .. }
            | SessionError::NewWindow { .. }
            | SessionError::Sidecar(_) => ErrorCode::Connect,
        };
        IpcError {
            code,
            message: message_with_source(&error),
            details: None,
        }
    }
}

/// The message of `error` with that of its source, which names the cause of
/// a failed start or I/O.
fn message_with_source(error: &SessionError) -> String {
    match std::error::Error::source(error) {
        Some(source) => format!("{error}: {source}"),
        None => error.to_string(),
    }
}
