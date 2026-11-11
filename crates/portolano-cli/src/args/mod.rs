//! Command-line arguments and global options (SPEC §10.1, §10.2).

mod add;
mod edit;
mod parse;

use std::path::PathBuf;

use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};
use portolano_ops::locations::VaultChoice;

pub use add::{
    AddCommand, AddCredArgs, AddFolderArgs, AddRdpArgs, AddSshArgs, CommonArgs, ConnectionArgs,
    PasswordArgs,
};
pub use edit::EditArgs;
pub use parse::{FieldArg, IconArg, TypeArg};

/// Portolano: gestore di connessioni SSH e RDP.
#[derive(Debug, Parser)]
#[command(name = "portolano", version, max_term_width = 100)]
pub struct Cli {
    /// Opzioni globali.
    #[command(flatten)]
    pub global: GlobalArgs,
    /// Il comando.
    #[command(subcommand)]
    pub command: Command,
}

/// Options valid for every command (SPEC §10.1).
#[derive(Debug, Clone, Args)]
pub struct GlobalArgs {
    /// Documento da usare.
    #[arg(long, global = true, env = "PORTOLANO_FILE", value_name = "PATH")]
    pub file: Option<PathBuf>,
    /// Vault dei segreti.
    #[arg(long, global = true, env = "PORTOLANO_VAULT", value_enum)]
    pub vault: Option<VaultArg>,
    /// File del vault cifrato (implica `--vault file`).
    #[arg(long, global = true, env = "PORTOLANO_VAULT_FILE", value_name = "PATH")]
    pub vault_file: Option<PathBuf>,
    /// File di configurazione.
    #[arg(long, global = true, value_name = "PATH")]
    pub config: Option<PathBuf>,
    /// Log più dettagliati su stderr (ripetibile).
    #[arg(short, long, global = true, action = ArgAction::Count)]
    pub verbose: u8,
    /// Output JSON (solo comandi di lettura).
    #[arg(long, global = true)]
    pub json: bool,
}

/// Vault backends selectable on the command line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum VaultArg {
    /// Portachiavi di sistema.
    Keychain,
    /// Vault su file cifrato.
    File,
    /// Vault in memoria (solo test).
    #[value(hide = true)]
    Memory,
}

impl From<VaultArg> for VaultChoice {
    fn from(value: VaultArg) -> Self {
        match value {
            VaultArg::Keychain => VaultChoice::Keychain,
            VaultArg::File => VaultChoice::File,
            VaultArg::Memory => VaultChoice::Memory,
        }
    }
}

/// The commands of SPEC §10.2.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Crea un documento vuoto.
    Init(InitArgs),
    /// Mostra l'albero.
    Tree(TreeArgs),
    /// Elenca i nodi in tabella.
    List(ListArgs),
    /// Mostra il dettaglio di un nodo.
    Show(ShowArgs),
    /// Aggiunge una cartella, una connessione o una credenziale.
    #[command(subcommand)]
    Add(AddCommand),
    /// Modifica un nodo.
    Edit(Box<EditArgs>),
    /// Sposta o riordina un nodo.
    Move(MoveArgs),
    /// Elimina un nodo.
    Delete(DeleteArgs),
    /// Gestisce i segreti di un nodo.
    #[command(subcommand)]
    Secret(SecretCommand),
    /// Apre una connessione.
    Connect(ConnectArgs),
    /// Importa un documento di un altro programma.
    Import(ImportArgs),
    /// Controlla il documento; esce con 4 se trova errori.
    Validate,
    /// Mostra o modifica la configurazione.
    #[command(subcommand)]
    Config(ConfigCommand),
    /// Client askpass per ssh (uso interno).
    #[command(hide = true)]
    Askpass {
        /// Il prompt di ssh.
        prompt: Option<String>,
    },
}

impl Command {
    /// Whether the command only reads, so that `--json` applies.
    pub fn is_read_only(&self) -> bool {
        matches!(
            self,
            Command::Tree(_)
                | Command::List(_)
                | Command::Show(_)
                | Command::Validate
                | Command::Secret(SecretCommand::Status { .. })
                | Command::Config(ConfigCommand::Show | ConfigCommand::Path)
        )
    }
}

/// `init`.
#[derive(Debug, Args)]
pub struct InitArgs {
    /// Nome del documento.
    #[arg(long, default_value = "Portolano")]
    pub name: String,
    /// Sostituisce un documento esistente (resta una copia in `.bak`).
    #[arg(long)]
    pub force: bool,
}

/// `tree`.
#[derive(Debug, Args)]
pub struct TreeArgs {
    /// Cartella da cui partire (default: la radice).
    pub path: Option<String>,
    /// Mostra l'id abbreviato di ogni nodo.
    #[arg(long)]
    pub ids: bool,
    /// Numero massimo di livelli.
    #[arg(long, value_name = "N")]
    pub depth: Option<usize>,
}

/// `list`.
#[derive(Debug, Args)]
pub struct ListArgs {
    /// Cartella da cui partire (default: la radice).
    pub path: Option<String>,
    /// Solo i nodi di questo tipo.
    #[arg(long = "type", value_enum)]
    pub node_type: Option<TypeArg>,
}

/// `show`.
#[derive(Debug, Args)]
pub struct ShowArgs {
    /// Il nodo.
    pub path: String,
    /// Aggiunge la credenziale effettiva (mai i segreti).
    #[arg(long)]
    pub resolved: bool,
}

/// `move`.
#[derive(Debug, Args)]
pub struct MoveArgs {
    /// Il nodo da spostare.
    pub path: String,
    /// La cartella di destinazione (`/` per la radice; il padre attuale per riordinare).
    pub destination: String,
    /// Posizione nella cartella, da 0.
    #[arg(long, value_name = "N", group = "placement")]
    pub position: Option<usize>,
    /// Prima di questo nodo della destinazione.
    #[arg(long, value_name = "PATH", group = "placement")]
    pub before: Option<String>,
    /// Dopo questo nodo della destinazione.
    #[arg(long, value_name = "PATH", group = "placement")]
    pub after: Option<String>,
}

/// `delete`.
#[derive(Debug, Args)]
pub struct DeleteArgs {
    /// Il nodo.
    pub path: String,
    /// Elimina anche il contenuto di una cartella.
    #[arg(short, long)]
    pub recursive: bool,
    /// Non chiede conferma.
    #[arg(short = 'y', long = "yes")]
    pub yes: bool,
    /// Elimina anche se altri nodi vi fanno riferimento.
    #[arg(long)]
    pub force: bool,
}

/// `secret`.
#[derive(Debug, Subcommand)]
pub enum SecretCommand {
    /// Salva un segreto nel vault.
    Set {
        /// Il nodo.
        path: String,
        /// Quale segreto: password, passphrase, key, prop:NOME.
        #[arg(long, default_value = "password")]
        field: FieldArg,
        /// Legge il segreto da stdin (una riga; tutto lo stdin per `key`).
        #[arg(long)]
        stdin: bool,
    },
    /// Elimina un segreto.
    Clear {
        /// Il nodo.
        path: String,
        /// Quale segreto: password, passphrase, key, prop:NOME.
        #[arg(long, default_value = "password")]
        field: FieldArg,
    },
    /// Dice quali segreti del nodo sono presenti nel vault.
    Status {
        /// Il nodo.
        path: String,
    },
}

/// `connect`.
#[derive(Debug, Args)]
pub struct ConnectArgs {
    /// La connessione.
    pub path: String,
    /// Apre in una nuova finestra del terminale.
    #[arg(long)]
    pub new_window: bool,
    /// Mostra il piano di lancio senza eseguirlo.
    #[arg(long)]
    pub print: bool,
    /// Client RDP da usare.
    #[arg(long, value_enum)]
    pub client: Option<ClientArg>,
    /// Attende la chiusura del client RDP.
    #[arg(long)]
    pub wait: bool,
}

/// RDP clients selectable with `--client`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ClientArg {
    /// FreeRDP 3.
    Freerdp,
    /// Microsoft Windows App.
    WindowsApp,
}

/// `import`.
#[derive(Debug, Args)]
pub struct ImportArgs {
    /// Il file da importare.
    #[arg(value_name = "FILE")]
    pub source: PathBuf,
    /// Formato del file (default: riconosciuto in automatico).
    #[arg(long)]
    pub format: Option<String>,
    /// Cartella di destinazione (default: la radice).
    #[arg(long, value_name = "PATH")]
    pub into: Option<String>,
    /// Cosa fare se un id importato esiste già.
    #[arg(long, value_enum, default_value = "fail")]
    pub on_conflict: ConflictArg,
    /// Chiede le password cifrate (invio vuoto per saltare).
    #[arg(long, conflicts_with = "dry_run")]
    pub prompt_secrets: bool,
    /// Importa anche il cestino.
    #[arg(long)]
    pub include_trash: bool,
    /// Salva il report in JSON.
    #[arg(long, value_name = "FILE")]
    pub report: Option<PathBuf>,
    /// Mostra il report senza scrivere nulla.
    #[arg(long)]
    pub dry_run: bool,
}

/// Values of `import --on-conflict`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ConflictArg {
    /// Errore con l'elenco dei conflitti.
    Fail,
    /// Sostituisce i nodi esistenti mantenendone posizione e segreti.
    Update,
    /// Importa con id nuovi.
    NewIds,
}

/// `config`.
#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    /// Mostra la configurazione.
    Show,
    /// Imposta una chiave (es. `rdp.client freerdp`).
    Set {
        /// La chiave: default_document o sezione.campo.
        key: String,
        /// Il valore.
        value: String,
    },
    /// Mostra il percorso del file di configurazione.
    Path,
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn argument_definitions_are_consistent() {
        Cli::command().debug_assert();
    }

    #[test]
    fn json_is_accepted_only_by_read_commands() {
        let read = Cli::try_parse_from(["portolano", "tree"]).unwrap();
        let write = Cli::try_parse_from(["portolano", "add", "folder", "/x"]).unwrap();
        assert!(read.command.is_read_only());
        assert!(!write.command.is_read_only());
    }
}
