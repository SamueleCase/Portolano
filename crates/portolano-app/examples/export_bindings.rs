//! Writes the TypeScript bindings to the path given as the only argument:
//! `cargo run -p portolano-app --example export_bindings -- <path>`
//! (`npm run bindings` in `frontend/`).

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let Some(path) = std::env::args_os().nth(1).map(PathBuf::from) else {
        eprintln!("uso: export_bindings <percorso di bindings.ts>");
        return ExitCode::from(2);
    };
    match portolano_app_lib::export_bindings(&path) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("esportazione dei bindings non riuscita: {error}");
            ExitCode::FAILURE
        }
    }
}
