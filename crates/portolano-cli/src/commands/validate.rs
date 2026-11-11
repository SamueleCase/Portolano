//! `portolano validate`: the problems of the document (SPEC §4.4).

use anyhow::Result;
use portolano_core::{Issue, Severity};
use portolano_ops::validate::{has_errors, validate};
use serde::Serialize;
use uuid::Uuid;

use crate::context::Context;
use crate::failure::EXIT_VALIDATION;
use crate::output::{print_json, visible};

/// An issue with the path of its node, for the output.
#[derive(Serialize)]
struct Located<'a> {
    #[serde(flatten)]
    issue: &'a Issue,
    path: Option<String>,
}

/// Prints the problems of the document, of its protocols and of the vault;
/// exits with 4 if there is an error (DEC-016.6).
///
/// # Errors
/// Missing document, vault errors.
pub fn run(ctx: &Context) -> Result<i32> {
    let (_, doc) = ctx.open()?;
    let vault = ctx.vault();
    let issues = validate(&doc, &ctx.protocols, Some(&vault))?;
    let path = |id: Option<Uuid>| id.and_then(|id| doc.path_of(id));
    if ctx.json {
        let located: Vec<Located> = issues
            .iter()
            .map(|issue| Located {
                issue,
                path: path(issue.node),
            })
            .collect();
        print_json(&located)?;
    } else {
        for issue in &issues {
            let line = format!(
                "{:<7} {}  [{}] {}",
                severity(issue.severity),
                path(issue.node).unwrap_or_else(|| "/".to_owned()),
                issue.code,
                issue.message
            );
            println!("{}", visible(&line));
        }
        let count = |wanted| issues.iter().filter(|i| i.severity == wanted).count();
        println!(
            "{} errori, {} avvisi, {} note",
            count(Severity::Error),
            count(Severity::Warning),
            count(Severity::Info)
        );
    }
    Ok(if has_errors(&issues) {
        EXIT_VALIDATION
    } else {
        0
    })
}

fn severity(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "errore",
        Severity::Warning => "avviso",
        Severity::Info => "nota",
    }
}
