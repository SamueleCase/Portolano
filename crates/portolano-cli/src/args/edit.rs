//! Arguments of `edit`.

use clap::Args;
use serde_json::Value;

use super::add::CommonArgs;
use super::parse::{CredArg, setting};

/// `edit`.
#[derive(Debug, Args)]
pub struct EditArgs {
    /// Il nodo.
    pub path: String,
    /// Nuovo nome.
    #[arg(long)]
    pub name: Option<String>,
    /// Nuovo host (connessioni).
    #[arg(long)]
    pub host: Option<String>,
    /// Nuova porta (connessioni).
    #[arg(long)]
    pub port: Option<u16>,
    /// Nuova credenziale: inherit, none, prompt, name:NOME oppure un percorso.
    #[arg(long, value_name = "SPEC")]
    pub cred: Option<CredArg>,
    /// Utente: della credenziale, o della credenziale inline (vuoto per cancellarlo).
    #[arg(long, conflicts_with = "cred")]
    pub user: Option<String>,
    /// Dominio: come `--user`.
    #[arg(long, conflicts_with = "cred")]
    pub domain: Option<String>,
    /// Imposta un'impostazione del protocollo CHIAVE=JSON (ripetibile).
    #[arg(long = "set", value_name = "K=JSON", value_parser = setting)]
    pub set: Vec<(String, Value)>,
    /// Toglie un'impostazione del protocollo (ripetibile).
    #[arg(long = "unset", value_name = "K")]
    pub unset: Vec<String>,
    /// Toglie un tag (ripetibile).
    #[arg(long = "untag", value_name = "T")]
    pub untag: Vec<String>,
    /// Toglie una proprietà (ripetibile).
    #[arg(long = "unprop", value_name = "K")]
    pub unprop: Vec<String>,
    /// Opzioni comuni.
    #[command(flatten)]
    pub common: CommonArgs,
}
