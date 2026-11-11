//! The text form of an import report (SPEC §8.2): names, paths and reasons
//! only.

use portolano_import::ItemStatus;
use portolano_ops::import::ImportResult;

use super::print_table;

/// Prints the report of an import and what was written.
pub fn print_import(result: &ImportResult, root_path: &str) {
    let report = &result.report;
    println!(
        "Import di '{}' ({}): documento '{}' in {root_path}",
        report.source_file, report.importer, report.document_name
    );
    let rows: Vec<Vec<String>> = report
        .counts
        .iter()
        .map(|(source_type, counts)| {
            vec![
                source_type.clone(),
                counts.source.to_string(),
                counts.imported.to_string(),
                counts.partial.to_string(),
                counts.unsupported.to_string(),
                counts.skipped.to_string(),
            ]
        })
        .collect();
    print_table(
        &[
            "TIPO",
            "ORIGINE",
            "IMPORTATI",
            "PARZIALI",
            "NON SUPPORTATI",
            "SALTATI",
        ],
        &rows,
    );
    if !report.items.is_empty() {
        println!("\nElementi non importati completamente:");
        for item in &report.items {
            println!(
                "  [{}] {} ({}): {}",
                status(item.status),
                item.path,
                item.source_type,
                item.reasons.join("; ")
            );
        }
    }
    if !report.unmapped_fields.is_empty() {
        println!("\nCampi non mappati (conservati in extra):");
        for (source_type, fields) in &report.unmapped_fields {
            let fields: Vec<String> = fields
                .iter()
                .map(|(field, count)| format!("{field} ({count})"))
                .collect();
            println!("  {source_type}: {}", fields.join(", "));
        }
    }
    for warning in report.warnings.iter().chain(&result.warnings) {
        println!("avviso: {warning}");
    }
    let verb = if result.dry_run {
        "Simulazione, nessuna modifica scritta"
    } else {
        "Fatto"
    };
    println!(
        "\n{verb}: {} nodi inseriti, {} aggiornati, {} segreti, {} icone.",
        result.inserted, result.updated, result.secrets_written, result.icons_stored
    );
}

fn status(status: ItemStatus) -> &'static str {
    match status {
        ItemStatus::Imported => "importato",
        ItemStatus::Partial => "parziale",
        ItemStatus::Unsupported => "non supportato",
        ItemStatus::Skipped => "saltato",
    }
}
