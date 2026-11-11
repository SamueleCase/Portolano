//! Parsers of option values with a syntax of their own.

use std::path::PathBuf;
use std::str::FromStr;

use clap::ValueEnum;
use portolano_connectors::rdp::Resolution;
use portolano_ops::inspect::TypeFilter;
use portolano_ops::nodes::CredentialInput;
use portolano_ops::secrets::SecretSlot;
use serde_json::Value;

/// `--cred`: `inherit` | `none` | `prompt` | `name:<NOME>` | `<PATH>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CredArg(pub CredentialInput);

impl FromStr for CredArg {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Ok(CredArg(match text {
            "inherit" => CredentialInput::Inherit,
            "none" => CredentialInput::None,
            "prompt" => CredentialInput::Prompt,
            "" => return Err("valore vuoto".to_owned()),
            _ => match text.strip_prefix("name:") {
                Some("") => return Err("nome della credenziale vuoto".to_owned()),
                Some(name) => CredentialInput::ByName(name.to_owned()),
                None => CredentialInput::ByPath(text.to_owned()),
            },
        }))
    }
}

/// `--field`: `password` | `passphrase` | `key` | `prop:<NOME>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldArg(pub SecretSlot);

impl FromStr for FieldArg {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Ok(FieldArg(match text {
            "password" => SecretSlot::Password,
            "passphrase" => SecretSlot::Passphrase,
            "key" => SecretSlot::PrivateKey,
            _ => match text.strip_prefix("prop:") {
                Some(name) if !name.is_empty() => SecretSlot::Property(name.to_owned()),
                _ => {
                    return Err("atteso password, passphrase, key oppure prop:NOME".to_owned());
                }
            },
        }))
    }
}

/// `--icon`: `builtin:N` | `royal:N` | `file:PATH` | `none`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IconArg {
    /// An icon of the Portolano set.
    Builtin(String),
    /// An icon of the Royal library.
    Royal(String),
    /// An image file.
    File(PathBuf),
    /// No icon.
    None,
}

impl FromStr for IconArg {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        if text == "none" {
            return Ok(IconArg::None);
        }
        let (kind, value) = text
            .split_once(':')
            .filter(|(_, value)| !value.is_empty())
            .ok_or("atteso builtin:NOME, royal:NOME, file:PERCORSO oppure none")?;
        match kind {
            "builtin" => Ok(IconArg::Builtin(value.to_owned())),
            "royal" => Ok(IconArg::Royal(value.to_owned())),
            "file" => Ok(IconArg::File(PathBuf::from(value))),
            _ => Err(format!("tipo di icona sconosciuto '{kind}'")),
        }
    }
}

/// `list --type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum TypeArg {
    /// Cartelle.
    Folder,
    /// Connessioni di ogni protocollo.
    Connection,
    /// Credenziali.
    Credential,
    /// Connessioni SSH.
    Ssh,
    /// Connessioni RDP.
    Rdp,
    /// Nodi non supportati.
    Unsupported,
}

impl From<TypeArg> for TypeFilter {
    fn from(value: TypeArg) -> Self {
        match value {
            TypeArg::Folder => TypeFilter::Folder,
            TypeArg::Connection => TypeFilter::Connection,
            TypeArg::Credential => TypeFilter::Credential,
            TypeArg::Ssh => TypeFilter::Protocol("ssh".to_owned()),
            TypeArg::Rdp => TypeFilter::Protocol("rdp".to_owned()),
            TypeArg::Unsupported => TypeFilter::Unsupported,
        }
    }
}

/// `K=V`, for `--prop` and `-o`.
pub fn key_value(text: &str) -> Result<(String, String), String> {
    match text.split_once('=') {
        Some((key, value)) if !key.is_empty() => Ok((key.to_owned(), value.to_owned())),
        _ => Err("atteso CHIAVE=VALORE".to_owned()),
    }
}

/// `K=JSON` for `edit --set`: a value that is not JSON is taken as a string.
pub fn setting(text: &str) -> Result<(String, Value), String> {
    let (key, raw) = key_value(text)?;
    let value = serde_json::from_str(&raw).unwrap_or(Value::String(raw));
    Ok((key, value))
}

/// `--resolution`: `fit` | `fullscreen` | `WxH`.
pub fn resolution(text: &str) -> Result<Resolution, String> {
    match text {
        "fit" => Ok(Resolution::Fit),
        "fullscreen" => Ok(Resolution::Fullscreen),
        _ => text
            .split_once(['x', 'X'])
            .and_then(|(width, height)| Some((width.parse().ok()?, height.parse().ok()?)))
            .map(|(width, height)| Resolution::Fixed { width, height })
            .ok_or_else(|| "atteso fit, fullscreen oppure LARGHEZZAxALTEZZA".to_owned()),
    }
}

/// `--gateway HOST[:PORT]`; an IPv6 address goes in brackets.
pub fn host_port(text: &str) -> Result<(String, Option<u16>), String> {
    let invalid = || "atteso HOST oppure HOST:PORTA".to_owned();
    if let Some(rest) = text.strip_prefix('[') {
        let (host, after) = rest.split_once(']').ok_or_else(invalid)?;
        let port = match after.strip_prefix(':') {
            Some(port) => Some(port.parse().map_err(|_| invalid())?),
            None if after.is_empty() => None,
            None => return Err(invalid()),
        };
        return Ok((host.to_owned(), port));
    }
    match text.rsplit_once(':') {
        Some((host, port)) if !host.contains(':') => {
            Ok((host.to_owned(), Some(port.parse().map_err(|_| invalid())?)))
        }
        _ if text.is_empty() => Err(invalid()),
        _ => Ok((text.to_owned(), None)),
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn cred_accepts_every_form_of_the_spec() {
        let parse = |text: &str| text.parse::<CredArg>().map(|arg| arg.0);
        assert_eq!(parse("inherit"), Ok(CredentialInput::Inherit));
        assert_eq!(parse("prompt"), Ok(CredentialInput::Prompt));
        assert_eq!(
            parse("name:Admin ACME"),
            Ok(CredentialInput::ByName("Admin ACME".to_owned()))
        );
        assert_eq!(
            parse("/Clienti/Admin"),
            Ok(CredentialInput::ByPath("/Clienti/Admin".to_owned()))
        );
        assert!(parse("name:").is_err());
    }

    #[test]
    fn field_accepts_properties() {
        assert_eq!(
            "prop:Token".parse::<FieldArg>().map(|arg| arg.0),
            Ok(SecretSlot::Property("Token".to_owned()))
        );
        assert!("pin".parse::<FieldArg>().is_err());
    }

    #[test]
    fn icon_forms_are_parsed() {
        assert_eq!(
            "royal:Flat/Hardware/Server".parse(),
            Ok(IconArg::Royal("Flat/Hardware/Server".to_owned()))
        );
        assert_eq!("none".parse(), Ok(IconArg::None));
        assert!("emoji:x".parse::<IconArg>().is_err());
    }

    #[test]
    fn setting_falls_back_to_a_string() {
        assert_eq!(
            setting("x11=true"),
            Ok(("x11".to_owned(), Value::Bool(true)))
        );
        assert_eq!(
            setting("note=ciao mondo"),
            Ok(("note".to_owned(), Value::String("ciao mondo".to_owned())))
        );
    }

    #[test]
    fn resolution_forms_are_parsed() {
        assert_eq!(
            resolution("1920x1080"),
            Ok(Resolution::Fixed {
                width: 1920,
                height: 1080
            })
        );
        assert!(resolution("grande").is_err());
    }

    #[test]
    fn host_port_handles_ports_and_ipv6() {
        assert_eq!(
            host_port("gw.example.com:443"),
            Ok(("gw.example.com".to_owned(), Some(443)))
        );
        assert_eq!(
            host_port("gw.example.com"),
            Ok(("gw.example.com".to_owned(), None))
        );
        assert_eq!(
            host_port("[2001:db8::1]:443"),
            Ok(("2001:db8::1".to_owned(), Some(443)))
        );
        assert!(host_port("gw.example.com:porta").is_err());
    }
}
