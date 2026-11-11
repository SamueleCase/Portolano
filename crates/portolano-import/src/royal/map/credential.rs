//! Credentials: how connections and folders obtain their login data, and
//! the `RoyalCredential` objects (SPEC §8.3.5, `docs/royal-format.md` §5).

use portolano_core::{Credential, CredentialSpec, PrivateKey};
use uuid::Uuid;

use super::context::{ObjectContext, SecretCollector};
use super::references::References;
use crate::{ImportError, SecretField};

/// [ipo] Passphrase of the private key of a `RoyalCredential`.
const PASSPHRASE: &str = "Passphrase";
/// [ipo] Private key embedded in a `RoyalCredential`.
const EMBEDDED_KEY: &str = "PrivateKeyContent";
/// Inline password of a connection or folder.
const INLINE_PASSWORD: &str = "CredentialPassword";

/// The credential of a connection or folder.
///
/// Only the password of the active mode is requested: with any other mode
/// Royal ignores a stored `CredentialPassword`, so it is dropped silently.
///
/// # Errors
/// The error of the prompter.
pub(super) fn spec(
    ctx: &mut ObjectContext<'_>,
    references: &References,
    secrets: &mut SecretCollector<'_>,
) -> Result<CredentialSpec, ImportError> {
    let from_parent = ctx.fields.take_flag("CredentialFromParent");
    let mode = ctx.fields.take("CredentialMode");
    let spec = match mode {
        _ if from_parent => CredentialSpec::Inherit,
        None => CredentialSpec::Inherit,
        Some("0") => CredentialSpec::None,
        Some("1") => CredentialSpec::Prompt,
        Some("2") => return inline(ctx, secrets),
        Some("3") => by_id(ctx, references),
        Some("4") => by_name(ctx),
        Some(other) => {
            ctx.add_reason(format!(
                "modalità credenziale Royal sconosciuta ({other}): la credenziale viene ereditata"
            ));
            CredentialSpec::Inherit
        }
    };
    ctx.fields.take_raw(INLINE_PASSWORD);
    Ok(spec)
}

fn inline(
    ctx: &mut ObjectContext<'_>,
    secrets: &mut SecretCollector<'_>,
) -> Result<CredentialSpec, ImportError> {
    let user = ctx.fields.take("CredentialUsername");
    let (username, domain) = split_user(user, None);
    ctx.username = user.map(str::to_owned);
    let password = secrets.request(ctx, INLINE_PASSWORD, SecretField::Password)?;
    Ok(CredentialSpec::Inline {
        username,
        domain,
        password,
    })
}

fn by_id(ctx: &mut ObjectContext<'_>, references: &References) -> CredentialSpec {
    let Some(id) = ctx
        .fields
        .take_with("CredentialId", |value| Uuid::parse_str(value).ok())
    else {
        ctx.add_reason("credenziale per ID senza un ID valido: la credenziale viene ereditata");
        return CredentialSpec::Inherit;
    };
    if !references.credentials.contains(&id) {
        ctx.add_reason(if references.in_file.contains(&id) {
            "riferimento a una credenziale non importata"
        } else {
            "riferimento a credenziale esterna al documento"
        });
    }
    CredentialSpec::ById { id }
}

fn by_name(ctx: &mut ObjectContext<'_>) -> CredentialSpec {
    match ctx.fields.take("CredentialName") {
        Some(name) => CredentialSpec::ByName {
            name: name.to_owned(),
        },
        None => {
            ctx.add_reason("credenziale per nome senza nome: la credenziale viene ereditata");
            CredentialSpec::Inherit
        }
    }
}

/// A `RoyalCredential`.
///
/// # Errors
/// The error of the prompter.
pub(super) fn credential(
    ctx: &mut ObjectContext<'_>,
    secrets: &mut SecretCollector<'_>,
) -> Result<Credential, ImportError> {
    let user = ctx.fields.take("UserName");
    let domain = ctx.fields.take("Domain");
    let (username, domain) = split_user(user, domain);
    ctx.username = user.map(str::to_owned);
    let password = secrets.request(ctx, "Password", SecretField::Password)?;
    let private_key = private_key(ctx, secrets)?;
    let passphrase = secrets.request(ctx, PASSPHRASE, SecretField::Passphrase)?;
    Ok(Credential {
        username,
        domain,
        password,
        private_key,
        passphrase,
    })
}

/// `PrivateKeyMode` 0 (or absent) means a key file; [ipo] any other value
/// means a key embedded in the document.
fn private_key(
    ctx: &mut ObjectContext<'_>,
    secrets: &mut SecretCollector<'_>,
) -> Result<Option<PrivateKey>, ImportError> {
    let embedded = ctx
        .fields
        .peek("PrivateKeyMode")
        .is_some_and(|mode| mode != "0");
    if !embedded {
        ctx.fields.take_raw("PrivateKeyMode");
        let path = ctx.fields.take("PrivateKeyPath");
        return Ok(path.map(|path| PrivateKey::File {
            path: path.to_owned(),
        }));
    }
    if ctx.fields.peek(EMBEDDED_KEY).is_none() {
        ctx.add_reason("chiave privata incorporata non trovata nel documento");
        return Ok(None);
    }
    ctx.fields.take_raw("PrivateKeyMode");
    let secret = secrets.request(ctx, EMBEDDED_KEY, SecretField::PrivateKey)?;
    Ok(secret.map(|secret| PrivateKey::Vault { secret }))
}

/// User name and domain: an explicit domain wins, otherwise `DOMAIN\user`
/// is split. `user@domain` stays as it is (SPEC §8.3.5).
fn split_user(user: Option<&str>, domain: Option<&str>) -> (Option<String>, Option<String>) {
    let split = match (user, domain) {
        (Some(user), None) => user
            .split_once('\\')
            .filter(|(domain, name)| !domain.is_empty() && !name.is_empty()),
        _ => None,
    };
    match split {
        Some((domain, name)) => (Some(name.to_owned()), Some(domain.to_owned())),
        None => (user.map(str::to_owned), domain.map(str::to_owned)),
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn owned(user: Option<&str>, domain: Option<&str>) -> (Option<String>, Option<String>) {
        (user.map(str::to_owned), domain.map(str::to_owned))
    }

    #[test]
    fn split_user_splits_domain_backslash_user() {
        assert_eq!(
            split_user(Some("ACME\\admin"), None),
            owned(Some("admin"), Some("ACME"))
        );
    }

    #[test]
    fn split_user_keeps_user_at_domain() {
        assert_eq!(
            split_user(Some("admin@acme.example.com"), None),
            owned(Some("admin@acme.example.com"), None)
        );
    }

    #[test]
    fn split_user_prefers_the_explicit_domain() {
        assert_eq!(
            split_user(Some("admin"), Some("ACME")),
            owned(Some("admin"), Some("ACME"))
        );
    }

    #[test]
    fn split_user_keeps_a_leading_or_trailing_backslash() {
        assert_eq!(
            split_user(Some("\\admin"), None),
            owned(Some("\\admin"), None)
        );
    }
}
