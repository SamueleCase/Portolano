use std::path::PathBuf;

use pretty_assertions::assert_eq;
use uuid::Uuid;

use super::*;

fn not_found() -> portolano_core::Error {
    portolano_core::Error::NotFound {
        query: "/Clienti/web01".to_owned(),
    }
}

fn locked() -> VaultError {
    VaultError::MasterPasswordUnavailable {
        reason: "vault bloccato".to_owned(),
    }
}

#[test]
fn lookup_without_match_is_not_found() {
    assert_eq!(
        OpsError::Core(not_found()).category(),
        ErrorCategory::NotFound
    );
}

#[test]
fn lookup_with_several_matches_is_ambiguous() {
    let error = OpsError::Core(portolano_core::Error::Ambiguous {
        query: "web".to_owned(),
        candidates: vec![(Uuid::from_u128(1), "/web01".to_owned())],
    });

    assert_eq!(error.category(), ErrorCategory::Ambiguous);
}

#[test]
fn unavailable_master_password_means_a_locked_vault() {
    assert_eq!(
        OpsError::Vault(locked()).category(),
        ErrorCategory::VaultLocked
    );
}

#[test]
fn wrong_master_password_is_a_vault_error() {
    assert_eq!(
        OpsError::Vault(VaultError::BadMasterPassword).category(),
        ErrorCategory::Vault
    );
}

#[test]
fn locked_vault_while_connecting_is_still_a_locked_vault() {
    let error = OpsError::Connect(ConnectError::Vault(locked()));

    assert_eq!(error.category(), ErrorCategory::VaultLocked);
}

#[test]
fn missing_node_to_connect_to_is_not_found() {
    let error = OpsError::Connect(ConnectError::Core(not_found()));

    assert_eq!(error.category(), ErrorCategory::NotFound);
}

#[test]
fn other_core_error_while_connecting_is_a_connect_error() {
    let error = OpsError::Connect(ConnectError::Core(portolano_core::Error::NoCredential {
        id: Uuid::from_u128(1),
    }));

    assert_eq!(error.category(), ErrorCategory::Connect);
}

#[test]
fn invalid_value_is_invalid() {
    assert_eq!(
        OpsError::InvalidPort { port: 0 }.category(),
        ErrorCategory::Invalid
    );
}

#[test]
fn name_already_taken_is_rejected() {
    let error = OpsError::NameTaken {
        parent: "/Clienti".to_owned(),
        name: "web01".to_owned(),
    };

    assert_eq!(error.category(), ErrorCategory::Rejected);
}

#[test]
fn options_that_exclude_each_other_are_a_usage_error() {
    assert_eq!(
        OpsError::KeyFileAndVaultKey.category(),
        ErrorCategory::Usage
    );
}

#[test]
fn document_changed_by_another_process_is_modified_externally() {
    let error = OpsError::Core(portolano_core::Error::ModifiedExternally {
        path: PathBuf::from("/tmp/portolano.json"),
    });

    assert_eq!(error.category(), ErrorCategory::ModifiedExternally);
}

#[test]
fn absent_document_is_missing() {
    let error = OpsError::DocumentNotFound {
        path: PathBuf::from("/tmp/portolano.json"),
    };

    assert_eq!(error.category(), ErrorCategory::DocumentMissing);
}

#[test]
fn import_file_in_a_broken_format_is_unreadable() {
    let error = OpsError::Import(ImportError::Xml {
        reason: "riga 1".to_owned(),
    });

    assert_eq!(error.category(), ErrorCategory::Unreadable);
}
