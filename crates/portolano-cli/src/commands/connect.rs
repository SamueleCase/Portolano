//! `portolano connect`: open a connection, or show its launch plan.

use std::path::Path;

use anyhow::Result;
use portolano_connectors::{ConnectOptions, Environment, RdpClient, prepare, render};
use portolano_ops::connect::{login_request, plan_connection};
use portolano_ops::vault::SessionVault;

use crate::args::{ClientArg, ConnectArgs};
use crate::context::Context;
use crate::prompt;

/// With `--print` prints the plan; otherwise asks the login data of a
/// credential "ask at launch", runs the plan and returns the exit code of
/// the client. The secrets are read from the vault only at that point.
///
/// # Errors
/// Path errors, planning errors (exit 6), vault errors.
pub fn run(ctx: &Context, args: ConnectArgs) -> Result<i32> {
    let (_, doc) = ctx.open()?;
    let id = doc.resolve_path(&args.path)?;
    let env = Environment::discover(&ctx.config)?;
    let mut options = ConnectOptions {
        new_window: args.new_window,
        rdp_client: args.client.map(|client| match client {
            ClientArg::Freerdp => RdpClient::FreeRdp,
            ClientArg::WindowsApp => RdpClient::WindowsApp,
        }),
        wait: args.wait,
        relaunch_args: ctx.relaunch_args(),
        session_credential: None,
    };
    if args.print {
        let plan = plan_connection(&doc, &ctx.config, &env, &ctx.protocols, id, &options)?;
        print_warnings(&plan.warnings);
        println!("{}", render(&plan));
        return Ok(0);
    }
    let vault = ctx.vault();
    let session = SessionVault::new(&vault);
    // In a new window the relaunched process asks, in that window.
    if !args.new_window
        && let Some(request) = login_request(&doc, id)?
        && let Some(login) = prompt::login(&request)?
    {
        options.session_credential = Some(session.session_credential(login)?);
    }
    let plan = plan_connection(&doc, &ctx.config, &env, &ctx.protocols, id, &options)?;
    print_warnings(&plan.warnings);
    let prepared = prepare(&plan, &session, &ctx.config)?;
    let log = prepared.log_path().map(Path::to_path_buf);
    let code = prepared.launch()?;
    if let Some(log) = log {
        eprintln!("log del client: {}", log.display());
    }
    Ok(code)
}

fn print_warnings(warnings: &[String]) {
    for warning in warnings {
        eprintln!("avviso: {warning}");
    }
}
