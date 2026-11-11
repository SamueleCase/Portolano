//! Launch plan of an SSH connection (SPEC §7.2): the jump chain, the
//! arguments of `ssh`, keys and passwords.

use std::path::PathBuf;

use portolano_core::{Connection, Document, PrivateKey, ResolvedCredential};
use uuid::Uuid;

use super::settings::{JumpHost, SshSettings};
use crate::ConnectError;
use crate::plan::{AskpassEntry, AskpassSpec, LaunchPlan, LaunchStep, PromptMatcher};
use crate::protocol::{check_value, host_port};

/// Port of SSH when the connection does not set one.
pub(super) const SSH_PORT: u16 = 22;

/// Characters that would split or end a `-J` entry.
const JUMP_FORBIDDEN: [char; 2] = [',', ' '];

/// Lifetime of a vault key in `ssh-agent` (`ssh-add -t`), in seconds: long
/// enough to log in, short enough not to linger.
const SSH_ADD_LIFETIME_S: u32 = 300;

/// A hop whose login data come from the document: the target or a jump
/// host given by reference.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Endpoint {
    /// The connection node, named by errors.
    pub(super) node: Uuid,
    /// Host name or address.
    pub(super) host: String,
    /// Port.
    pub(super) port: u16,
    /// Resolved login data.
    pub(super) credential: ResolvedCredential,
}

/// A jump host, in `-J` order.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum Jump {
    /// A connection of the document.
    Endpoint(Endpoint),
    /// A literal `[user@]host[:port]`.
    Address(String),
}

/// A jump host of the chain, before its credential is resolved.
pub(super) enum JumpLink<'a> {
    /// An `ssh` connection of the document.
    Node {
        /// The connection node.
        id: Uuid,
        /// Its connection data.
        connection: &'a Connection,
    },
    /// A literal address.
    Address(String),
}

/// Everything [`build`] needs, already resolved.
pub(super) struct SshLaunch<'a> {
    /// The `ssh` program.
    pub(super) ssh: PathBuf,
    /// Whether `ssh-add` is installed, needed for keys in the vault.
    pub(super) ssh_add_available: bool,
    /// The destination.
    pub(super) target: Endpoint,
    /// The flattened jump chain.
    pub(super) jumps: Vec<Jump>,
    /// The settings of the destination.
    pub(super) settings: &'a SshSettings,
}

/// The flattened jump chain of a connection: the jumps of a jump come
/// before it.
///
/// # Errors
/// [`ConnectError::JumpCycle`] if the chain loops back to a node on the
/// current path, [`ConnectError::InvalidJumpHost`] if a reference does not
/// point to a valid `ssh` connection, [`ConnectError::InvalidValue`] for an
/// address with a comma, a space or a control character.
pub(super) fn jump_chain<'a>(
    doc: &'a Document,
    owner: Uuid,
    settings: &SshSettings,
) -> Result<Vec<JumpLink<'a>>, ConnectError> {
    let mut chain = Vec::new();
    follow(doc, owner, settings, &mut vec![owner], &mut chain)?;
    Ok(chain)
}

fn follow<'a>(
    doc: &'a Document,
    owner: Uuid,
    settings: &SshSettings,
    path: &mut Vec<Uuid>,
    chain: &mut Vec<JumpLink<'a>>,
) -> Result<(), ConnectError> {
    for jump in &settings.jump_hosts {
        match jump {
            JumpHost::Address { address } => {
                check_value(owner, "jump_hosts.address", address, &JUMP_FORBIDDEN)?;
                chain.push(JumpLink::Address(address.clone()));
            }
            JumpHost::Ref { id } => {
                let id = *id;
                if path.contains(&id) {
                    return Err(ConnectError::JumpCycle { id });
                }
                let connection = jump_connection(doc, id)?;
                let jump_settings = SshSettings::parse(&connection.settings).map_err(|reason| {
                    ConnectError::InvalidJumpHost {
                        id,
                        reason: format!("impostazioni non valide: {reason}"),
                    }
                })?;
                path.push(id);
                follow(doc, id, &jump_settings, path, chain)?;
                path.pop();
                chain.push(JumpLink::Node { id, connection });
            }
        }
    }
    Ok(())
}

fn jump_connection(doc: &Document, id: Uuid) -> Result<&Connection, ConnectError> {
    let invalid = |reason: String| ConnectError::InvalidJumpHost { id, reason };
    let node = doc
        .find(id)
        .ok_or_else(|| invalid("il nodo non esiste".to_owned()))?;
    match node.as_connection() {
        Some(connection) if connection.protocol == "ssh" => Ok(connection),
        Some(connection) => Err(invalid(format!(
            "è una connessione {}, non ssh",
            connection.protocol
        ))),
        None => Err(invalid("non è una connessione".to_owned())),
    }
}

/// Builds the plan that runs `ssh` in the current terminal.
///
/// # Errors
/// [`ConnectError::ClientNotFound`] if a key in the vault needs `ssh-add`
/// and it is not installed; [`ConnectError::InvalidValue`] for a jump host
/// whose user or host would break the `-J` list.
pub(super) fn build(launch: &SshLaunch) -> Result<LaunchPlan, ConnectError> {
    let mut secrets = HopSecrets::new(launch.ssh_add_available);
    let target = &launch.target;
    let mut args = vec!["-p".to_owned(), target.port.to_string()];
    if let Some(user) = &target.credential.username {
        args.extend(["-l".to_owned(), user.clone()]);
    }
    if let Some(key_path) = secrets.add_key(target, Role::Target)? {
        args.extend(["-i".to_owned(), key_path]);
        args.extend(["-o".to_owned(), "IdentitiesOnly=yes".to_owned()]);
    }
    secrets.add_password(target);
    if !launch.jumps.is_empty() {
        args.extend(["-J".to_owned(), jump_list(&launch.jumps, &mut secrets)?]);
    }
    args.extend(flags(launch.settings));
    if !secrets.entries.is_empty() {
        // One attempt per prompt: askpass serves each secret once, and a
        // rejected password must not fall into repeated prompts.
        args.extend(["-o".to_owned(), "NumberOfPasswordPrompts=1".to_owned()]);
    }
    for (key, value) in &launch.settings.options {
        args.extend(["-o".to_owned(), format!("{key}={value}")]);
    }
    // `--` ends the options: a host taken from a document can never be read
    // as an option of ssh.
    args.extend(["--".to_owned(), target.host.clone()]);

    let askpass = (!secrets.entries.is_empty()).then_some(AskpassSpec {
        entries: secrets.entries,
    });
    let mut steps = secrets.ssh_add;
    steps.push(LaunchStep::Run {
        program: launch.ssh.clone(),
        args,
        env: Vec::new(),
        stdin_secret: None,
        askpass,
        wait: true,
    });
    Ok(LaunchPlan {
        steps,
        warnings: secrets.warnings,
    })
}

/// The value of `-J`, collecting the secrets of the jump hosts.
fn jump_list(jumps: &[Jump], secrets: &mut HopSecrets) -> Result<String, ConnectError> {
    let mut hops = Vec::with_capacity(jumps.len());
    for jump in jumps {
        match jump {
            Jump::Address(address) => hops.push(address.clone()),
            Jump::Endpoint(endpoint) => {
                check_value(endpoint.node, "host", &endpoint.host, &JUMP_FORBIDDEN)?;
                if let Some(user) = &endpoint.credential.username {
                    check_value(endpoint.node, "username", user, &JUMP_FORBIDDEN)?;
                }
                secrets.add_key(endpoint, Role::Jump)?;
                secrets.add_password(endpoint);
                hops.push(jump_address(endpoint));
            }
        }
    }
    Ok(hops.join(","))
}

fn flags(settings: &SshSettings) -> Vec<String> {
    let mut flags = Vec::new();
    if settings.agent_forwarding {
        flags.push("-A".to_owned());
    }
    if settings.x11_trusted {
        flags.push("-Y".to_owned());
    } else if settings.x11 {
        flags.push("-X".to_owned());
    }
    if settings.compression {
        flags.push("-C".to_owned());
    }
    if let Some(seconds) = settings.keepalive_s {
        flags.extend(["-o".to_owned(), format!("ServerAliveInterval={seconds}")]);
    }
    flags
}

/// `user@host:port` for `-J`.
fn jump_address(endpoint: &Endpoint) -> String {
    let user = endpoint
        .credential
        .username
        .as_ref()
        .map(|user| format!("{user}@"))
        .unwrap_or_default();
    format!("{user}{}", host_port(&endpoint.host, endpoint.port))
}

/// Where a hop sits in the connection.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Role {
    Target,
    Jump,
}

/// The secrets of the hops, as plan steps and askpass entries.
struct HopSecrets {
    ssh_add_available: bool,
    ssh_add: Vec<LaunchStep>,
    entries: Vec<AskpassEntry>,
    warnings: Vec<String>,
}

impl HopSecrets {
    fn new(ssh_add_available: bool) -> HopSecrets {
        HopSecrets {
            ssh_add_available,
            ssh_add: Vec::new(),
            entries: Vec::new(),
            warnings: Vec::new(),
        }
    }

    fn add_password(&mut self, hop: &Endpoint) {
        let Some(password) = &hop.credential.password else {
            return;
        };
        // The prompt names the user, so without one the entry could not be
        // matched safely.
        match &hop.credential.username {
            Some(user) => self.entries.push(AskpassEntry {
                matcher: PromptMatcher::Password {
                    user: user.clone(),
                    host: hop.host.clone(),
                },
                secret: password.clone(),
            }),
            None => self.warnings.push(format!(
                "la credenziale di {} ha una password ma nessun utente: \
                 la password salvata non viene usata e ssh la chiederà",
                hop.host
            )),
        }
    }

    /// Adds the key of `hop`; returns the expanded path of a key file of
    /// the target, which goes into `-i`.
    fn add_key(&mut self, hop: &Endpoint, role: Role) -> Result<Option<String>, ConnectError> {
        match &hop.credential.private_key {
            None => Ok(None),
            Some(PrivateKey::Vault { secret }) => {
                if !self.ssh_add_available {
                    return Err(ConnectError::ClientNotFound {
                        client: "ssh-add".to_owned(),
                        hint: "serve per le chiavi salvate nel vault: installa OpenSSH".to_owned(),
                    });
                }
                // The key goes to ssh-agent through stdin, never to a file.
                self.ssh_add.push(LaunchStep::SshAddKey {
                    key: secret.clone(),
                    passphrase: hop.credential.passphrase.clone(),
                    lifetime_s: SSH_ADD_LIFETIME_S,
                });
                Ok(None)
            }
            Some(PrivateKey::File { path }) if role == Role::Target => {
                let path = expand_home(path);
                if let Some(passphrase) = &hop.credential.passphrase {
                    self.entries.push(AskpassEntry {
                        matcher: PromptMatcher::Passphrase {
                            key_path: path.clone(),
                        },
                        secret: passphrase.clone(),
                    });
                }
                Ok(Some(path))
            }
            Some(PrivateKey::File { path }) => {
                // ProxyJump does not forward `-i` to the jump connection.
                self.warnings.push(format!(
                    "la chiave '{path}' del jump host {} non viene passata a ssh -J: \
                     aggiungila a ssh-agent o a ~/.ssh/config",
                    hop.host
                ));
                Ok(None)
            }
        }
    }
}

/// Expands a leading `~` to the home directory, as ssh prints the expanded
/// path in its passphrase prompt.
fn expand_home(path: &str) -> String {
    let rest = if path == "~" {
        Some("")
    } else {
        path.strip_prefix("~/")
    };
    match (rest, std::env::home_dir()) {
        (Some(""), Some(home)) => home.to_string_lossy().into_owned(),
        (Some(rest), Some(home)) => home.join(rest).to_string_lossy().into_owned(),
        _ => path.to_owned(),
    }
}

#[cfg(test)]
mod tests;
