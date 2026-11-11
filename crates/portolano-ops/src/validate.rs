//! Composed validation (SPEC §4.4, §10.2 `validate`): the structural checks
//! of the core, the checks of each protocol, unknown protocols and secrets
//! missing from the vault.

use portolano_connectors::Registry;
use portolano_core::{Document, Issue, Node, Severity, Vault};

use crate::Result;
use crate::tree_util::own_secret_refs;

/// Code of a connection whose protocol is not registered (Warning).
pub const UNKNOWN_PROTOCOL: &str = "unknown_protocol";

/// Code of a secret reference without an entry in the vault. A Warning: the
/// document is consistent, only the local vault lacks the entry, as on a
/// second machine that shares the document (DEC-022.4).
pub const MISSING_SECRET: &str = "missing_secret";

/// Every problem of the document. The vault is checked only if given, so
/// that a locked vault can be skipped.
///
/// # Errors
/// Errors of the vault.
pub fn validate(
    doc: &Document,
    protocols: &Registry,
    vault: Option<&dyn Vault>,
) -> Result<Vec<Issue>> {
    let mut issues = doc.validate();
    for (_, node) in doc.walk() {
        issues.extend(protocol_issues(doc, node, protocols));
        if let Some(vault) = vault {
            issues.extend(missing_secrets(doc, node, vault)?);
        }
    }
    Ok(issues)
}

/// Whether `issues` contain an error; warnings and notes do not count
/// (DEC-016.6).
pub fn has_errors(issues: &[Issue]) -> bool {
    issues.iter().any(|issue| issue.severity == Severity::Error)
}

fn protocol_issues(doc: &Document, node: &Node, protocols: &Registry) -> Vec<Issue> {
    let Some(connection) = node.as_connection() else {
        return Vec::new();
    };
    match protocols.get(&connection.protocol) {
        Some(protocol) => protocol.validate(doc, node),
        None => vec![Issue {
            severity: Severity::Warning,
            node: Some(node.id),
            code: UNKNOWN_PROTOCOL,
            message: format!(
                "la connessione '{}' usa il protocollo sconosciuto '{}'",
                node.name, connection.protocol
            ),
        }],
    }
}

fn missing_secrets(doc: &Document, node: &Node, vault: &dyn Vault) -> Result<Vec<Issue>> {
    let mut issues = Vec::new();
    for reference in own_secret_refs(doc, node) {
        if !vault.contains(&reference)? {
            issues.push(Issue {
                severity: Severity::Warning,
                node: Some(node.id),
                code: MISSING_SECRET,
                message: format!(
                    "il segreto {} del nodo '{}' non è nel vault ({})",
                    reference.as_str(),
                    node.name,
                    vault.backend()
                ),
            });
        }
    }
    Ok(issues)
}

#[cfg(test)]
mod tests {
    use portolano_core::vault::MemoryVault;
    use portolano_core::{CredentialSpec, SecretRef};
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::testutil::*;

    fn codes(issues: &[Issue]) -> Vec<&'static str> {
        issues.iter().map(|issue| issue.code).collect()
    }

    #[test]
    fn unknown_protocol_is_a_warning() {
        let doc = document(vec![connection("legacy", "telnet")]);
        let issues = validate(&doc, &Registry::with_defaults(), None).unwrap();
        assert_eq!(codes(&issues), vec![UNKNOWN_PROTOCOL]);
        assert_eq!(issues[0].severity, Severity::Warning);
        assert!(!has_errors(&issues));
    }

    #[test]
    fn protocol_checks_are_included() {
        let mut node = connection("web01", "ssh");
        if let Some(connection) = node.as_connection_mut() {
            connection
                .settings
                .insert("unknown_option".to_owned(), json!(true));
        }
        let doc = document(vec![node]);
        let issues = validate(&doc, &Registry::with_defaults(), None).unwrap();
        assert_eq!(codes(&issues), vec!["invalid_settings"]);
        assert!(has_errors(&issues));
    }

    #[test]
    fn core_checks_are_included() {
        let doc = document(vec![connection_with(
            "web01",
            "ssh",
            CredentialSpec::ByName {
                name: "Nessuna".to_owned(),
            },
        )]);
        let issues = validate(&doc, &Registry::with_defaults(), None).unwrap();
        assert_eq!(codes(&issues), vec!["by_name_unresolved"]);
    }

    #[test]
    fn secret_missing_from_the_vault_is_a_warning_on_its_node() {
        let (vault, present) = vault_with(SENTINEL);
        let doc = document(vec![folder(
            "Clienti",
            vec![
                credential("Ok", "admin", Some(present)),
                credential("Manca", "admin", Some(SecretRef::new())),
            ],
        )]);
        let issues = validate(&doc, &Registry::with_defaults(), Some(&vault)).unwrap();
        assert_eq!(codes(&issues), vec![MISSING_SECRET]);
        assert_eq!(issues[0].node, Some(id_of(&doc, "/Clienti/Manca")));
        assert_eq!(issues[0].severity, Severity::Warning);
    }

    #[test]
    fn vault_is_not_checked_when_absent() {
        let doc = document(vec![credential("Manca", "admin", Some(SecretRef::new()))]);
        let issues = validate(&doc, &Registry::with_defaults(), None).unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn present_secrets_give_no_issue() {
        let vault = MemoryVault::default();
        let doc = document(vec![credential("Admin", "admin", None)]);
        let issues = validate(&doc, &Registry::with_defaults(), Some(&vault)).unwrap();
        assert!(issues.is_empty());
    }
}
