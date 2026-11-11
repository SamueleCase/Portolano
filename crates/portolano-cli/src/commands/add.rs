//! `portolano add`: add folders, connections and credentials.

use std::collections::BTreeMap;

use anyhow::Result;
use portolano_connectors::rdp::{RdpGateway, RdpSettings};
use portolano_connectors::ssh::SshSettings;
use portolano_core::Document;
use portolano_ops::nodes::{
    CredentialInput, JumpInput, NewNode, NewNodeKind, NodeSecrets, add_node,
};
use serde::Serialize;
use serde_json::{Map, Value};

use super::common::{details, password};
use crate::args::{
    AddCommand, AddCredArgs, AddFolderArgs, AddRdpArgs, AddSshArgs, CommonArgs, ConnectionArgs,
};
use crate::context::Context;
use crate::output::visible;
use crate::prompt;

/// Adds the node and writes its secrets.
///
/// # Errors
/// Invalid input, path errors, vault and store errors.
pub fn run(ctx: &Context, command: AddCommand) -> Result<i32> {
    let (store, mut doc) = ctx.open()?;
    let node = match command {
        AddCommand::Folder(args) => {
            let AddFolderArgs {
                path,
                parents,
                common,
            } = *args;
            new_node(
                path,
                parents,
                NewNodeKind::Folder,
                NodeSecrets::default(),
                common,
            )?
        }
        AddCommand::Ssh(args) => ssh(&doc, *args)?,
        AddCommand::Rdp(args) => rdp(&doc, *args)?,
        AddCommand::Cred(args) => cred(*args)?,
    };
    let vault = ctx.vault();
    let outcome = add_node(&store, &mut doc, &vault, &ctx.protocols, node)?;
    println!("Aggiunto {}", visible(&outcome.path));
    Ok(0)
}

fn new_node(
    path: String,
    create_parents: bool,
    kind: NewNodeKind,
    mut secrets: NodeSecrets,
    common: CommonArgs,
) -> Result<NewNode> {
    let (details, properties) = details(common)?;
    secrets.properties = properties;
    Ok(NewNode {
        path,
        create_parents,
        kind,
        details,
        secrets,
    })
}

fn ssh(doc: &Document, args: AddSshArgs) -> Result<NewNode> {
    let mut options: BTreeMap<String, String> = args.options.into_iter().collect();
    // DEC-022.2: an inline credential has no key, so a key file becomes an
    // option of ssh.
    if let Some(key) = args.key {
        options.insert("IdentityFile".to_owned(), key);
        options.insert("IdentitiesOnly".to_owned(), "yes".to_owned());
    }
    let settings = SshSettings {
        jump_hosts: args
            .jump
            .into_iter()
            .map(|jump| JumpInput(jump).resolve(doc))
            .collect::<Result<_, _>>()?,
        agent_forwarding: args.agent_forwarding,
        x11: args.x11,
        x11_trusted: false,
        compression: args.compression,
        keepalive_s: args.keepalive,
        options,
    };
    connection(
        "ssh",
        args.connection,
        None,
        to_map(&settings)?,
        args.common,
    )
}

fn rdp(doc: &Document, args: AddRdpArgs) -> Result<NewNode> {
    let gateway = match args.gateway {
        Some((host, port)) => Some(RdpGateway {
            host,
            port,
            credential: args
                .gateway_cred
                .map(|cred| cred.0.resolve(doc))
                .transpose()?
                .unwrap_or_default(),
        }),
        None => None,
    };
    let settings = RdpSettings {
        resolution: args.resolution.unwrap_or_default(),
        scale: args.scale,
        color_depth: args.color_depth,
        gateway,
        multimon: args.multimon,
        admin: args.admin,
        ..RdpSettings::default()
    };
    connection(
        "rdp",
        args.connection,
        args.domain,
        to_map(&settings)?,
        args.common,
    )
}

fn connection(
    protocol: &str,
    args: ConnectionArgs,
    domain: Option<String>,
    settings: Map<String, Value>,
    common: CommonArgs,
) -> Result<NewNode> {
    let credential = match (args.cred, args.user, domain) {
        (Some(cred), _, _) => cred.0,
        (None, None, None) => CredentialInput::Inherit,
        (None, username, domain) => CredentialInput::Inline { username, domain },
    };
    let secrets = NodeSecrets {
        password: password(&args.password)?,
        ..NodeSecrets::default()
    };
    let kind = NewNodeKind::Connection {
        protocol: protocol.to_owned(),
        host: args.host,
        port: args.port,
        credential,
        settings,
    };
    new_node(args.path, args.parents, kind, secrets, common)
}

fn cred(args: AddCredArgs) -> Result<NewNode> {
    let secrets = NodeSecrets {
        password: password(&args.password)?,
        private_key: args.key_stdin.then(prompt::stdin_all).transpose()?,
        passphrase: args
            .ask_passphrase
            .then(|| prompt::new_secret("Passphrase della chiave"))
            .transpose()?,
        properties: Vec::new(),
    };
    let kind = NewNodeKind::Credential {
        username: args.user,
        domain: args.domain,
        key_file: args.key,
    };
    new_node(args.path, args.parents, kind, secrets, args.common)
}

/// Typed settings as the JSON map stored in the connection.
fn to_map(settings: &impl Serialize) -> Result<Map<String, Value>> {
    match serde_json::to_value(settings)? {
        Value::Object(map) => Ok(map),
        _ => Ok(Map::new()),
    }
}
