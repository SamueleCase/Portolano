//! The text form of `show` (SPEC §10.2): never a secret, only whether it is
//! set or present in the vault.

use portolano_connectors::Registry;
use portolano_core::{CredentialSpec, Document, Icon, Node, NodeKind, PrivateKey, PropertyValue};
use portolano_ops::inspect::{CredentialView, KeyView, OriginView, SecretState};

use super::{display_width, pad, short_id, visible};

/// The effective credential of `show --resolved`, or why it cannot be
/// resolved.
pub type Resolved = Option<Result<CredentialView, String>>;

/// Prints the details of `node`.
pub fn print_show(doc: &Document, node: &Node, protocols: &Registry, resolved: &Resolved) {
    let mut lines = Lines::default();
    lines.add("Nome", &node.name);
    lines.add("Percorso", &doc.path_of(node.id).unwrap_or_default());
    lines.add("Id", &format!("{} (@{})", node.id, short_id(node.id)));
    kind_lines(&mut lines, doc, node, protocols);
    common_lines(&mut lines, node);
    lines.print();
    match resolved {
        None => {}
        Some(Ok(view)) => {
            println!("Credenziale effettiva:");
            resolved_lines(view).print_indented();
        }
        Some(Err(reason)) => println!(
            "Credenziale effettiva: non risolvibile: {}",
            visible(reason)
        ),
    }
}

/// Labelled values, printed with aligned labels; every value is made
/// [`visible`].
#[derive(Default)]
struct Lines(Vec<(String, String)>);

impl Lines {
    /// A one-line value: a line break in it is escaped too.
    fn add(&mut self, label: &str, value: &str) {
        self.add_text(label, &visible(value));
    }

    /// A value that may span several lines (notes, settings): its line
    /// breaks are kept, the other control characters escaped when printed.
    fn add_text(&mut self, label: &str, value: &str) {
        self.0.push((visible(label).into_owned(), value.to_owned()));
    }

    fn add_some(&mut self, label: &str, value: Option<&str>) {
        if let Some(value) = value {
            self.add(label, value);
        }
    }

    fn print(&self) {
        self.print_with("");
    }

    fn print_indented(&self) {
        self.print_with("  ");
    }

    fn print_with(&self, indent: &str) {
        let width = self
            .0
            .iter()
            .map(|(label, _)| display_width(label) + 1)
            .max()
            .unwrap_or(0);
        for (label, value) in &self.0 {
            let mut values = value.lines();
            let first = values.next().unwrap_or_default();
            println!(
                "{indent}{} {}",
                pad(&format!("{label}:"), width),
                visible(first)
            );
            for more in values {
                println!("{indent}{} {}", " ".repeat(width), visible(more));
            }
        }
    }
}

fn kind_lines(lines: &mut Lines, doc: &Document, node: &Node, protocols: &Registry) {
    match &node.kind {
        NodeKind::Folder(folder) => {
            lines.add("Tipo", "cartella");
            lines.add("Elementi", &folder.children.len().to_string());
            lines.add("Credenziale", &describe_spec(doc, &folder.credential));
        }
        NodeKind::Connection(connection) => {
            lines.add("Tipo", &format!("connessione {}", connection.protocol));
            lines.add("Host", &connection.host);
            let port = match (connection.port, protocols.get(&connection.protocol)) {
                (Some(port), _) => port.to_string(),
                (None, Some(protocol)) => format!("{} (predefinita)", protocol.default_port()),
                (None, None) => "predefinita".to_owned(),
            };
            lines.add("Porta", &port);
            lines.add("Credenziale", &describe_spec(doc, &connection.credential));
            if !connection.settings.is_empty() {
                let settings =
                    serde_json::to_string_pretty(&connection.settings).unwrap_or_default();
                lines.add_text("Impostazioni", &settings);
            }
        }
        NodeKind::Credential(credential) => {
            lines.add("Tipo", "credenziale");
            lines.add_some("Utente", credential.username.as_deref());
            lines.add_some("Dominio", credential.domain.as_deref());
            lines.add("Password", set_or_not(credential.password.is_some()));
            match &credential.private_key {
                Some(PrivateKey::File { path }) => lines.add("Chiave", &format!("file {path}")),
                Some(PrivateKey::Vault { .. }) => lines.add("Chiave", "nel vault"),
                None => {}
            }
            if credential.passphrase.is_some() {
                lines.add("Passphrase", "impostata");
            }
        }
        NodeKind::Unsupported(unsupported) => {
            lines.add(
                "Tipo",
                &format!("non supportato ({})", unsupported.source_type),
            );
            lines.add(
                "Originale",
                &format!("{} byte conservati", unsupported.raw.len()),
            );
        }
    }
}

fn common_lines(lines: &mut Lines, node: &Node) {
    lines.add_some("Descrizione", node.description.as_deref());
    if let Some(notes) = &node.notes {
        lines.add_text("Note", notes);
    }
    if let Some(icon) = &node.icon {
        lines.add("Icona", &describe_icon(icon));
    }
    lines.add_some("Colore", node.color.as_deref());
    if node.favorite {
        lines.add("Preferito", "sì");
    }
    if !node.tags.is_empty() {
        lines.add("Tag", &node.tags.join(", "));
    }
    for property in &node.custom_properties {
        let value = match &property.value {
            PropertyValue::Text { value } => value.clone(),
            PropertyValue::Secret { .. } => "(segreto)".to_owned(),
        };
        lines.add(&format!("Proprietà {}", property.name), &value);
    }
    if let Some(source) = &node.source {
        let id = source.source_id.as_deref().unwrap_or("-");
        let file = source.file.as_deref().unwrap_or("-");
        lines.add(
            "Origine",
            &format!(
                "{} {} (id {id}, file {file})",
                source.system, source.source_type
            ),
        );
    }
    if !node.extra.is_empty() {
        lines.add(
            "Extra",
            &format!("{} campi importati non mappati", node.extra.len()),
        );
    }
    lines.add("Creato", &node.created.to_rfc3339());
    lines.add("Modificato", &node.modified.to_rfc3339());
}

fn resolved_lines(view: &CredentialView) -> Lines {
    let mut lines = Lines::default();
    let origin = match &view.origin {
        OriginView::None => "nessuna".to_owned(),
        OriginView::Prompt => "chiesta all'avvio".to_owned(),
        OriginView::Inline { path, .. } => format!("inline su {path}"),
        OriginView::Credential { path, .. } => format!("credenziale {path}"),
    };
    lines.add("Origine", &origin);
    lines.add_some("Utente", view.username.as_deref());
    lines.add_some("Dominio", view.domain.as_deref());
    lines.add("Password", describe_state(view.password));
    match &view.private_key {
        Some(KeyView::File { path }) => lines.add("Chiave", &format!("file {path}")),
        Some(KeyView::Vault { state }) => lines.add("Chiave", describe_state(*state)),
        None => {}
    }
    if view.passphrase != SecretState::NotSet {
        lines.add("Passphrase", describe_state(view.passphrase));
    }
    lines
}

fn describe_state(state: SecretState) -> &'static str {
    match state {
        SecretState::NotSet => "non impostata",
        SecretState::Present => "presente nel vault",
        SecretState::Absent => "assente nel vault",
    }
}

fn set_or_not(set: bool) -> &'static str {
    if set { "impostata" } else { "non impostata" }
}

fn describe_spec(doc: &Document, spec: &CredentialSpec) -> String {
    match spec {
        CredentialSpec::Inherit => "ereditata".to_owned(),
        CredentialSpec::None => "nessuna".to_owned(),
        CredentialSpec::Prompt => "chiesta all'avvio".to_owned(),
        CredentialSpec::Inline {
            username,
            domain,
            password,
        } => {
            let mut parts = Vec::new();
            if let Some(username) = username {
                parts.push(format!("utente {username}"));
            }
            if let Some(domain) = domain {
                parts.push(format!("dominio {domain}"));
            }
            parts.push(format!("password {}", set_or_not(password.is_some())));
            format!("inline ({})", parts.join(", "))
        }
        CredentialSpec::ById { id } => match doc.path_of(*id) {
            Some(path) => format!("credenziale {path}"),
            None => format!("credenziale @{id} (inesistente)"),
        },
        CredentialSpec::ByName { name } => format!("per nome '{name}'"),
    }
}

fn describe_icon(icon: &Icon) -> String {
    match icon {
        Icon::Builtin { name } => format!("builtin:{name}"),
        Icon::Royal { name } => format!("royal:{name}"),
        Icon::Custom { sha256, mime } => format!("immagine {mime} ({sha256})"),
    }
}
