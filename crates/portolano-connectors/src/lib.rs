//! Portolano connectors (SPEC §7).
//!
//! A [`Protocol`] turns a connection node into a pure [`LaunchPlan`];
//! [`execute`] runs the plan, reading secrets from the vault only at that
//! point. Secrets never appear in argv, environment variables or files.

mod decode;
mod env;
mod exec;
mod plan;
mod protocol;
mod registry;
mod terminal;

pub mod rdp;
pub mod ssh;

#[cfg(test)]
mod testutil;

#[cfg(test)]
mod secrecy_tests;

use std::path::PathBuf;

use portolano_core::{SecretRef, VaultError};
use uuid::Uuid;

pub use env::{Environment, Os};
pub use exec::{PreparedLaunch, execute, prepare};
pub use plan::{AskpassEntry, AskpassSpec, LaunchPlan, LaunchStep, PromptMatcher, render};
pub use protocol::{ConnectOptions, PlanContext, Protocol, SessionCredential};
pub use rdp::RdpClient;
pub use registry::Registry;

/// Errors raised before or while launching a connection.
///
/// Messages are in Italian, name the node or program involved and never
/// contain a secret.
#[derive(Debug, thiserror::Error)]
pub enum ConnectError {
    /// A core error: missing node, credential resolution, …
    #[error(transparent)]
    Core(#[from] portolano_core::Error),

    /// A vault error while reading the secrets of the plan.
    #[error(transparent)]
    Vault(#[from] VaultError),

    /// The node is not a connection.
    #[error("il nodo {id} non è una connessione")]
    NotAConnection {
        /// The node.
        id: Uuid,
    },

    /// The connection uses a protocol that is not registered.
    #[error("il nodo {id} usa il protocollo sconosciuto '{protocol}'")]
    UnknownProtocol {
        /// The connection.
        id: Uuid,
        /// The `protocol` field of the connection.
        protocol: String,
    },

    /// The `settings` of a connection do not match its protocol.
    #[error("impostazioni non valide per il nodo {id}: {reason}")]
    InvalidSettings {
        /// The connection.
        id: Uuid,
        /// What is wrong.
        reason: String,
    },

    /// A protocol section of `config.toml` is invalid.
    #[error("sezione [{section}] della configurazione non valida: {reason}")]
    InvalidConfig {
        /// The section name, such as `"rdp"`.
        section: String,
        /// What is wrong.
        reason: String,
    },

    /// A jump host reference does not point to an SSH connection.
    #[error("jump host {id} non valido: {reason}")]
    InvalidJumpHost {
        /// The referenced node.
        id: Uuid,
        /// What is wrong.
        reason: String,
    },

    /// The chain of jump hosts loops.
    #[error("ciclo nella catena dei jump host: il nodo {id} compare più volte")]
    JumpCycle {
        /// A node met twice while following the chain.
        id: Uuid,
    },

    /// No suitable client program is installed.
    #[error("{client} non trovato: {hint}")]
    ClientNotFound {
        /// The missing client, such as `"FreeRDP"`.
        client: String,
        /// How to install it, such as `"brew install freerdp"`.
        hint: String,
    },

    /// A key in the vault needs `ssh-agent`, which is not available.
    #[error("ssh-agent non disponibile: serve per usare una chiave salvata nel vault")]
    AgentUnavailable,

    /// A secret required by the plan is not in the vault.
    #[error("segreto {} assente nel vault", .reference.as_str())]
    MissingSecret {
        /// The missing entry.
        reference: SecretRef,
    },

    /// A program could not be started.
    #[error("avvio di '{program}' non riuscito")]
    Spawn {
        /// The program.
        program: PathBuf,
        /// The underlying error.
        #[source]
        source: std::io::Error,
    },

    /// A filesystem or socket operation failed.
    #[error("errore di I/O: {context}")]
    Io {
        /// What was being done, such as "creazione della directory temporanea".
        context: String,
        /// The underlying error.
        #[source]
        source: std::io::Error,
    },

    /// The askpass server failed.
    #[error("askpass: {reason}")]
    Askpass {
        /// What went wrong.
        reason: String,
    },

    /// A feature is not available on this platform.
    #[error("{feature} non è supportato su questa piattaforma")]
    UnsupportedPlatform {
        /// The feature, such as "l'apertura in una nuova finestra".
        feature: String,
    },
}
