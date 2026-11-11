//! Users and hosts that may reach the command line of `ssh` (DEC-024.3,
//! DEC-026.2). ssh may expand them into commands (`%h`, `%r` in a
//! `ProxyCommand` or `Match exec` of `~/.ssh/config`); OpenSSH from 9.6
//! (CVE-2023-51385) refuses such users and hosts itself, and these checks
//! protect older versions too.

use uuid::Uuid;

use super::Endpoint;
use crate::ConnectError;
use crate::protocol::check_value;

/// Characters that would split or end a `-J` entry, or that a shell would
/// interpret: OpenSSH may turn the jump hosts into a proxy command run by
/// the shell, so quotes, substitutions and separators are refused too.
const JUMP_FORBIDDEN: [char; 14] = [
    ',', ' ', '\'', '"', '`', ';', '$', '|', '&', '<', '>', '(', ')', '\\',
];

/// Characters of the shell refused in the user of the destination, as
/// OpenSSH's `valid_ruser` does, plus `$`.
const USER_FORBIDDEN: [char; 13] = [
    '\'', '`', '"', ';', '&', '<', '>', '|', '(', ')', '{', '}', '$',
];

/// Checks a jump host given as an address, field `jump_hosts.address` of
/// the node `owner`.
///
/// # Errors
/// [`ConnectError::InvalidValue`] for a control character or one of
/// [`JUMP_FORBIDDEN`].
pub(super) fn check_jump_address(owner: Uuid, address: &str) -> Result<(), ConnectError> {
    check_value(owner, "jump_hosts.address", address, &JUMP_FORBIDDEN)
}

/// Checks the host and the user of the destination: the host with the rule
/// of the jump hosts, the user with [`check_target_user`].
///
/// # Errors
/// [`ConnectError::InvalidValue`] naming the field.
pub(super) fn check_target(target: &Endpoint) -> Result<(), ConnectError> {
    check_value(target.node, "host", &target.host, &JUMP_FORBIDDEN)?;
    if let Some(user) = &target.credential.username {
        check_target_user(target.node, user)?;
    }
    Ok(())
}

/// Refuses a host or user of a jump host with a control character or one
/// of [`JUMP_FORBIDDEN`]: the stricter rule of `-J`, whose entries are
/// separated by commas.
///
/// # Errors
/// [`ConnectError::InvalidValue`] naming the field.
pub(super) fn check_jump(hop: &Endpoint) -> Result<(), ConnectError> {
    check_value(hop.node, "host", &hop.host, &JUMP_FORBIDDEN)?;
    if let Some(user) = &hop.credential.username {
        check_value(hop.node, "username", user, &JUMP_FORBIDDEN)?;
    }
    Ok(())
}

/// Checks the user of the destination with the rule of OpenSSH
/// (`valid_ruser`, DEC-026.2): no shell metacharacter nor control character,
/// no leading `-` nor `-` after a space (it would read as an option), no
/// trailing `\`. A space, or a `\` as in `DOMINIO\utente`, is fine: `-l`
/// passes the user as one argument.
fn check_target_user(node: Uuid, user: &str) -> Result<(), ConnectError> {
    check_value(node, "username", user, &USER_FORBIDDEN)?;
    let invalid = |reason: &str| ConnectError::InvalidValue {
        id: node,
        field: "username".to_owned(),
        reason: reason.to_owned(),
    };
    if user.starts_with('-') {
        return Err(invalid("inizia con '-'"));
    }
    if user
        .char_indices()
        .any(|(index, c)| c.is_whitespace() && user[index + c.len_utf8()..].starts_with('-'))
    {
        return Err(invalid("contiene '-' dopo uno spazio"));
    }
    if user.ends_with('\\') {
        return Err(invalid("finisce con '\\'"));
    }
    Ok(())
}
