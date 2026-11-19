use std::io;
use std::path::PathBuf;

use portolano_core::VaultError;
use pretty_assertions::assert_eq;
use serde_json::json;

use super::*;

#[test]
fn error_serializes_with_camel_case_code_and_without_empty_details() {
    let error = IpcError {
        code: ErrorCode::ModifiedExternally,
        message: "documento modificato da un altro processo".to_owned(),
        details: None,
    };

    let json = serde_json::to_value(&error).unwrap();

    assert_eq!(
        json,
        json!({ "code": "modifiedExternally", "message": "documento modificato da un altro processo" })
    );
}

#[test]
fn a_change_not_confirmed_serializes_as_cancelled_without_details() {
    let json = serde_json::to_value(IpcError::cancelled("modifica annullata")).unwrap();

    assert_eq!(
        json,
        json!({ "code": "cancelled", "message": "modifica annullata" })
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
fn ambiguous_node_to_connect_to_carries_its_candidates() {
    let id = Uuid::from_u128(9);
    let error = IpcError::from(OpsError::Connect(ConnectError::Core(
        portolano_core::Error::Ambiguous {
            query: "web".to_owned(),
            candidates: vec![(id, "/Clienti/web01".to_owned())],
        },
    )));

    assert_eq!(error.code, ErrorCode::Ambiguous);
    assert_eq!(
        error.details,
        Some(IpcErrorDetails::Ambiguous {
            candidates: vec![Candidate {
                id,
                path: "/Clienti/web01".to_owned()
            }]
        })
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

/// The error of `settings` refused by the ssh protocol of the registry.
fn refused_ssh_settings(settings: serde_json::Value) -> IpcError {
    let registry = portolano_connectors::Registry::with_defaults();
    let ssh = registry.get("ssh").unwrap();
    let reason = ssh
        .validate_settings(settings.as_object().unwrap())
        .unwrap_err();
    IpcError::from(OpsError::InvalidSettings {
        path: "/web01".to_owned(),
        reason,
    })
}

#[test]
fn refused_ssh_option_names_the_setting_and_the_option() {
    let error = refused_ssh_settings(json!({ "options": { "ProxyCommand": "nc %h %p" } }));

    assert_eq!(
        error.details,
        Some(IpcErrorDetails::Invalid {
            field: "settings.options.ProxyCommand".to_owned()
        })
    );
}

#[test]
fn refused_ssh_option_keeps_the_message_of_the_cli() {
    let error = refused_ssh_settings(json!({ "options": { "ProxyCommand": "nc %h %p" } }));

    assert!(
        error
            .message
            .starts_with("impostazioni non valide per '/web01': campo 'options'"),
        "{}",
        error.message
    );
}

#[test]
fn refused_jump_host_names_its_setting() {
    let error = refused_ssh_settings(json!({ "jump_hosts": [{ "host": "b" }] }));

    assert_eq!(
        error.details,
        Some(IpcErrorDetails::Invalid {
            field: "settings.jump_hosts".to_owned()
        })
    );
}

/// The field of the error of settings refused for `reason`.
fn settings_field_of(reason: SettingsError) -> Option<IpcErrorDetails> {
    IpcError::from(OpsError::InvalidSettings {
        path: "/web01".to_owned(),
        reason,
    })
    .details
}

#[test]
fn refused_rdp_value_names_its_setting() {
    let registry = portolano_connectors::Registry::with_defaults();
    let rdp = registry.get("rdp").unwrap();
    let reason = rdp
        .validate_settings(json!({ "scale": 150 }).as_object().unwrap())
        .unwrap_err();

    assert_eq!(
        settings_field_of(reason),
        Some(IpcErrorDetails::Invalid {
            field: "settings.scale".to_owned()
        })
    );
}

#[test]
fn settings_refused_as_a_whole_name_the_settings() {
    let reason = SettingsError {
        setting: None,
        option: None,
        message: "impostazioni non valide".to_owned(),
    };

    assert_eq!(
        settings_field_of(reason),
        Some(IpcErrorDetails::Invalid {
            field: "settings".to_owned()
        })
    );
}

#[test]
fn image_missing_from_the_icon_store_is_refused_on_the_icon_field() {
    let error = IpcError::from(OpsError::IconNotStored);

    assert_eq!(error.code, ErrorCode::Invalid);
    assert_eq!(
        error.details,
        Some(IpcErrorDetails::Invalid {
            field: "icon".to_owned()
        })
    );
}
