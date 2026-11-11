//! Arguments of `add` and the options shared with `edit`.

use std::path::PathBuf;

use clap::{Args, Subcommand};
use portolano_connectors::rdp::Resolution;

use super::parse::{CredArg, IconArg, host_port, key_value, resolution};

/// `add`.
#[derive(Debug, Subcommand)]
pub enum AddCommand {
    /// Aggiunge una cartella.
    Folder(Box<AddFolderArgs>),
    /// Aggiunge una connessione SSH.
    Ssh(Box<AddSshArgs>),
    /// Aggiunge una connessione RDP.
    Rdp(Box<AddRdpArgs>),
    /// Aggiunge una credenziale.
    Cred(Box<AddCredArgs>),
}

/// `add folder`.
#[derive(Debug, Args)]
pub struct AddFolderArgs {
    /// Percorso della nuova cartella.
    pub path: String,
    /// Crea le cartelle intermedie mancanti.
    #[arg(short = 'p', long = "parents")]
    pub parents: bool,
    /// Opzioni comuni.
    #[command(flatten)]
    pub common: CommonArgs,
}

/// `--ask-password` / `--password-stdin` (there is no `--password <value>`).
#[derive(Debug, Clone, Args)]
pub struct PasswordArgs {
    /// Chiede la password (input nascosto, con conferma).
    #[arg(long, conflicts_with = "password_stdin")]
    pub ask_password: bool,
    /// Legge la password da una riga di stdin.
    #[arg(long)]
    pub password_stdin: bool,
}

/// Where a new connection points and how it logs in.
#[derive(Debug, Args)]
pub struct ConnectionArgs {
    /// Percorso della nuova connessione.
    pub path: String,
    /// Host.
    #[arg(long)]
    pub host: String,
    /// Porta (default: quella del protocollo).
    #[arg(long)]
    pub port: Option<u16>,
    /// Utente (senza `--cred` crea una credenziale inline).
    #[arg(long, conflicts_with = "cred")]
    pub user: Option<String>,
    /// Credenziale: inherit, none, prompt, name:NOME oppure il percorso di una credenziale
    /// (la password di una credenziale collegata si imposta su quella).
    #[arg(long, value_name = "SPEC", conflicts_with_all = ["ask_password", "password_stdin"])]
    pub cred: Option<CredArg>,
    /// Password della credenziale inline.
    #[command(flatten)]
    pub password: PasswordArgs,
    /// Crea le cartelle intermedie mancanti.
    #[arg(short = 'p', long = "parents")]
    pub parents: bool,
}

/// `add ssh`.
#[derive(Debug, Args)]
pub struct AddSshArgs {
    /// Destinazione e credenziale.
    #[command(flatten)]
    pub connection: ConnectionArgs,
    /// File della chiave privata. Va nelle opzioni ssh (IdentityFile, IdentitiesOnly):
    /// `show --resolved` non lo mostra come chiave della credenziale; per una chiave
    /// riusabile usa `add cred --key` e `--cred`.
    #[arg(long, value_name = "FILE")]
    pub key: Option<String>,
    /// Jump host: percorso di una connessione ssh oppure [utente@]host[:porta] (ripetibile).
    #[arg(long, value_name = "PATH|ADDR")]
    pub jump: Vec<String>,
    /// Inoltro dell'agent (-A).
    #[arg(long)]
    pub agent_forwarding: bool,
    /// Inoltro X11 (-X).
    #[arg(long)]
    pub x11: bool,
    /// Compressione (-C).
    #[arg(long)]
    pub compression: bool,
    /// Keepalive in secondi (ServerAliveInterval).
    #[arg(long, value_name = "S")]
    pub keepalive: Option<u32>,
    /// Opzione ssh -o CHIAVE=VALORE (ripetibile).
    #[arg(short = 'o', value_name = "K=V", value_parser = key_value)]
    pub options: Vec<(String, String)>,
    /// Opzioni comuni.
    #[command(flatten)]
    pub common: CommonArgs,
}

/// `add rdp`.
#[derive(Debug, Args)]
pub struct AddRdpArgs {
    /// Destinazione e credenziale.
    #[command(flatten)]
    pub connection: ConnectionArgs,
    /// Dominio Windows della credenziale inline.
    #[arg(long, conflicts_with = "cred")]
    pub domain: Option<String>,
    /// Risoluzione: fit, fullscreen oppure LARGHEZZAxALTEZZA.
    #[arg(long, value_parser = resolution)]
    pub resolution: Option<Resolution>,
    /// Scala: 100, 140 o 180.
    #[arg(long, value_name = "N")]
    pub scale: Option<u32>,
    /// Profondità di colore: 15, 16, 24 o 32.
    #[arg(long, value_name = "N")]
    pub color_depth: Option<u8>,
    /// Remote Desktop Gateway.
    #[arg(long, value_name = "HOST[:PORT]", value_parser = host_port)]
    pub gateway: Option<(String, Option<u16>)>,
    /// Credenziale del gateway, come `--cred`.
    #[arg(long, value_name = "SPEC", requires = "gateway")]
    pub gateway_cred: Option<CredArg>,
    /// Usa tutti i monitor.
    #[arg(long)]
    pub multimon: bool,
    /// Sessione amministrativa.
    #[arg(long)]
    pub admin: bool,
    /// Opzioni comuni.
    #[command(flatten)]
    pub common: CommonArgs,
}

/// `add cred`.
#[derive(Debug, Args)]
pub struct AddCredArgs {
    /// Percorso della nuova credenziale.
    pub path: String,
    /// Utente.
    #[arg(long)]
    pub user: Option<String>,
    /// Dominio Windows.
    #[arg(long)]
    pub domain: Option<String>,
    /// Password.
    #[command(flatten)]
    pub password: PasswordArgs,
    /// File della chiave privata.
    #[arg(long, value_name = "FILE", conflicts_with = "key_stdin")]
    pub key: Option<String>,
    /// Legge la chiave privata da stdin e la salva nel vault.
    #[arg(long, conflicts_with = "password_stdin")]
    pub key_stdin: bool,
    /// Chiede la passphrase della chiave.
    #[arg(long)]
    pub ask_passphrase: bool,
    /// Crea le cartelle intermedie mancanti.
    #[arg(short = 'p', long = "parents")]
    pub parents: bool,
    /// Opzioni comuni.
    #[command(flatten)]
    pub common: CommonArgs,
}

/// Options shared by `add` and `edit` (SPEC §10.2).
#[derive(Debug, Clone, Args)]
pub struct CommonArgs {
    /// Descrizione (vuota per cancellarla).
    #[arg(long)]
    pub description: Option<String>,
    /// Note (vuote per cancellarle).
    #[arg(long, conflicts_with = "notes_file")]
    pub notes: Option<String>,
    /// Legge le note da un file.
    #[arg(long, value_name = "FILE")]
    pub notes_file: Option<PathBuf>,
    /// Icona: builtin:NOME, royal:NOME, file:PERCORSO oppure none.
    #[arg(long)]
    pub icon: Option<IconArg>,
    /// Colore #RRGGBB (vuoto per cancellarlo).
    #[arg(long)]
    pub color: Option<String>,
    /// Segna come preferito.
    #[arg(long, conflicts_with = "no_favorite")]
    pub favorite: bool,
    /// Toglie dai preferiti.
    #[arg(long)]
    pub no_favorite: bool,
    /// Aggiunge un tag (ripetibile).
    #[arg(long = "tag", value_name = "T")]
    pub tags: Vec<String>,
    /// Imposta una proprietà testuale CHIAVE=VALORE (ripetibile).
    #[arg(long = "prop", value_name = "K=V", value_parser = key_value)]
    pub props: Vec<(String, String)>,
    /// Chiede il valore di una proprietà segreta (ripetibile).
    #[arg(long = "secret-prop", value_name = "K")]
    pub secret_props: Vec<String>,
}
