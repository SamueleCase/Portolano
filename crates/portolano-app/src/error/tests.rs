use std::io;
use std::path::PathBuf;

use portolano_core::VaultError;
use pretty_assertions::assert_eq;
use serde_json::json;

use super::*;

#[test]
fn error_serializes_with_camel_case_code_and_without_empty_details() {
    let error = IpcError {
        code: ErrorCode::NoDocument,
        message: "nessun documento aperto".to_owned(),
        details: None,
    };

    let json = serde_json::to_value(&error).unwrap();

    assert_eq!(
        json,
        json!({ "code": "noDocument", "message": "nessun documento aperto" })
    );
}

#[test]
fn locked_vault_keeps_its_own_code() {
    let error = IpcError::from(OpsError::Vault(VaultError::MasterPasswordUnavailable {
        reason: "vault bloccato".to_owned(),
    }));

    assert_eq!(error.code, ErrorCode::VaultLocked);
}

#[test]
fn ambiguous_lookup_carries_its_candidates() {
    let id = Uuid::from_u128(7);
    let error = IpcError::from(OpsError::Core(portolano_core::Error::Ambiguous {
        query: "web".to_owned(),
        candidates: vec![(id, "/Clienti/web01".to_owned())],
    }));

    let json = serde_json::to_value(&error).unwrap();

    assert_eq!(json["code"], "ambiguous");
    assert_eq!(
        json["details"],
        json!({ "kind": "ambiguous", "candidates": [{ "id": id, "path": "/Clienti/web01" }] })
    );
}

#[test]
fn refused_value_names_its_field() {
    let error = IpcError::from(OpsError::InvalidPort { port: 0 });

    assert_eq!(error.code, ErrorCode::Invalid);
    assert_eq!(
        error.details,
        Some(IpcErrorDetails::Invalid {
            field: "port".to_owned()
        })
    );
}

#[test]
fn referenced_node_carries_the_references() {
    let reference = ExternalReference {
        from: Uuid::from_u128(1),
        from_path: "/web01".to_owned(),
        target: Uuid::from_u128(2),
        target_path: "/Admin".to_owned(),
    };
    let error = IpcError::from(OpsError::Referenced {
        path: "/Admin".to_owned(),
        references: vec![reference.clone()],
    });

    assert_eq!(error.code, ErrorCode::Referenced);
    assert_eq!(
        error.details,
        Some(IpcErrorDetails::Referenced {
            references: vec![reference]
        })
    );
}

#[test]
fn message_includes_the_cause_of_an_io_error() {
    let error = IpcError::from(OpsError::Core(portolano_core::Error::Io {
        path: PathBuf::from("/tmp/portolano.json"),
        source: io::Error::new(io::ErrorKind::PermissionDenied, "permesso negato"),
    }));

    assert_eq!(error.code, ErrorCode::Io);
    assert_eq!(
        error.message,
        "errore di I/O su '/tmp/portolano.json': permesso negato"
    );
}
