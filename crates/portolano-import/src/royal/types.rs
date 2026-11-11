//! Names of the Royal object types the importer recognizes
//! (`docs/royal-format.md` §4).

/// The logical root of the document.
pub(super) const DOCUMENT: &str = "RoyalDocument";
/// The trash: its content is skipped unless requested.
pub(super) const TRASH: &str = "RoyalTrash";
/// A folder.
pub(super) const FOLDER: &str = "RoyalFolder";
/// An SSH connection; also Telnet and serial, by flag.
pub(super) const SSH_CONNECTION: &str = "RoyalSSHConnection";
/// An RDP connection.
pub(super) const RDS_CONNECTION: &str = "RoyalRDSConnection";
/// Reusable login data.
pub(super) const CREDENTIAL: &str = "RoyalCredential";
/// An SSH tunnel used as jump host by other connections.
pub(super) const SECURE_GATEWAY: &str = "RoyalSecureGateway";
/// A reusable Remote Desktop Gateway.
pub(super) const RDS_GATEWAY: &str = "RoyalRDSGateway";

/// Whether objects of this type can contain other objects.
pub(super) fn is_container(type_name: &str) -> bool {
    matches!(type_name, DOCUMENT | FOLDER | TRASH)
}
