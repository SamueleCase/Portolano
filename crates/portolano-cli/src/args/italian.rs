//! Help and usage errors in Italian (DEC-023.2): clap writes them in
//! English, so the command is adjusted before parsing (headings, help and
//! version flags) and the main errors are rewritten, keeping exit code 2.

use std::num::{IntErrorKind, ParseIntError};

use clap::error::{ContextKind, ErrorKind};
use clap::{Arg, ArgAction, Command, CommandFactory, FromArgMatches};

use super::Cli;
use crate::failure::EXIT_USAGE;
use crate::output::visible;

/// What clap writes between a number and the range it is not in.
const CLAP_OUT_OF_RANGE: &str = " is not in ";

/// Parses the process arguments. On a usage error, or after printing the
/// help or the version, returns the exit code instead.
pub fn parse() -> Result<Cli, i32> {
    let matches = command()
        .try_get_matches()
        .map_err(|error| report(&error))?;
    Cli::from_arg_matches(&matches).map_err(|error| report(&error))
}

/// The command of [`Cli`] with Italian headings and help flags.
pub fn command() -> Command {
    let version = Arg::new("version")
        .short('V')
        .long("version")
        .action(ArgAction::Version)
        .help("Mostra la versione");
    localize(Cli::command())
        .disable_version_flag(true)
        .arg(version)
}

fn localize(command: Command) -> Command {
    let template = template(&command);
    let help = Arg::new("help")
        .short('h')
        .long("help")
        .action(ArgAction::Help)
        .help("Mostra l'aiuto");
    command
        .disable_help_flag(true)
        .disable_help_subcommand(true)
        .arg(help)
        .help_template(template)
        .mut_subcommands(localize)
}

/// Headings in Italian; only the sections the command has.
fn template(command: &Command) -> String {
    let mut template = String::from("{about-with-newline}\nUso: {usage}\n");
    if command.has_subcommands() {
        template.push_str("\nComandi:\n{subcommands}\n");
    }
    if command.get_positionals().next().is_some() {
        template.push_str("\nArgomenti:\n{positionals}\n");
    }
    template.push_str("\nOpzioni:\n{options}\n{after-help}");
    template
}

/// Prints the help, the version or the error, and returns the exit code.
fn report(error: &clap::Error) -> i32 {
    match error.kind() {
        ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
            let _ = error.print();
            0
        }
        ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
            let _ = error.print();
            EXIT_USAGE
        }
        _ => {
            eprintln!("errore: {}", visible(&message(error)));
            eprintln!("\nPer l'aiuto: portolano --help, oppure portolano <comando> --help");
            EXIT_USAGE
        }
    }
}

/// The Italian message of a usage error.
fn message(error: &clap::Error) -> String {
    let context = |kind| error.get(kind).map(ToString::to_string).unwrap_or_default();
    let suggestion = |kind| {
        error
            .get(kind)
            .map(|value| format!(" (forse intendevi '{value}'?)"))
            .unwrap_or_default()
    };
    match error.kind() {
        ErrorKind::MissingRequiredArgument => {
            format!(
                "manca l'argomento obbligatorio {}",
                context(ContextKind::InvalidArg)
            )
        }
        ErrorKind::MissingSubcommand => "manca il comando".to_owned(),
        ErrorKind::InvalidSubcommand => format!(
            "comando sconosciuto '{}'{}",
            context(ContextKind::InvalidSubcommand),
            suggestion(ContextKind::SuggestedSubcommand)
        ),
        ErrorKind::UnknownArgument => format!(
            "argomento inatteso '{}'{}",
            context(ContextKind::InvalidArg),
            suggestion(ContextKind::SuggestedArg)
        ),
        ErrorKind::InvalidValue => {
            let mut text = format!(
                "valore non valido '{}' per {}",
                context(ContextKind::InvalidValue),
                context(ContextKind::InvalidArg)
            );
            if let Some(valid) = error.get(ContextKind::ValidValue) {
                text.push_str(&format!("; valori ammessi: {valid}"));
            }
            text
        }
        ErrorKind::ValueValidation => {
            let reason = std::error::Error::source(error)
                .map(|source| format!(": {}", reason(source)))
                .unwrap_or_default();
            format!(
                "valore non valido '{}' per {}{reason}",
                context(ContextKind::InvalidValue),
                context(ContextKind::InvalidArg)
            )
        }
        ErrorKind::ArgumentConflict => format!(
            "{} non si può usare insieme a {}",
            context(ContextKind::InvalidArg),
            context(ContextKind::PriorArg)
        ),
        ErrorKind::TooManyValues | ErrorKind::WrongNumberOfValues => format!(
            "numero di valori errato per {}",
            context(ContextKind::InvalidArg)
        ),
        _ => "uso errato degli argomenti".to_owned(),
    }
}

/// Why a value was rejected: our parsers already explain it in Italian;
/// the numeric parsers of the standard library and of clap in English.
fn reason(source: &(dyn std::error::Error + 'static)) -> String {
    let Some(error) = source.downcast_ref::<ParseIntError>() else {
        let text = source.to_string();
        // Message of clap's integer parser: "<value> is not in <range>".
        return match text.split_once(CLAP_OUT_OF_RANGE) {
            Some((_, range)) => format!("valore fuori intervallo ({range})"),
            None => text,
        };
    };
    match error.kind() {
        IntErrorKind::Empty => "valore vuoto",
        IntErrorKind::InvalidDigit => "cifra non valida",
        IntErrorKind::PosOverflow | IntErrorKind::NegOverflow => "valore fuori intervallo",
        IntErrorKind::Zero => "lo zero non è ammesso",
        _ => "numero non valido",
    }
    .to_owned()
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn error_of(args: &[&str]) -> String {
        let error = command().try_get_matches_from(args).unwrap_err();
        message(&error)
    }

    #[test]
    fn localized_command_is_consistent() {
        command().debug_assert();
    }

    #[test]
    fn missing_argument_is_explained_in_italian() {
        assert_eq!(
            error_of(&["portolano", "add", "ssh", "/web01"]),
            "manca l'argomento obbligatorio --host <HOST>"
        );
    }

    #[test]
    fn unknown_command_suggests_the_closest() {
        assert_eq!(
            error_of(&["portolano", "conect"]),
            "comando sconosciuto 'conect' (forse intendevi 'connect'?)"
        );
    }

    #[test]
    fn invalid_value_lists_the_accepted_ones() {
        assert_eq!(
            error_of(&["portolano", "--vault", "cassaforte", "tree"]),
            "valore non valido 'cassaforte' per --vault <VAULT>; valori ammessi: keychain, file"
        );
    }

    #[test]
    fn conflicting_options_are_named() {
        let text = error_of(&[
            "portolano",
            "add",
            "cred",
            "/a",
            "--ask-password",
            "--password-stdin",
        ]);
        assert!(text.contains("non si può usare insieme a"), "{text}");
    }

    #[test]
    fn help_has_italian_headings() {
        let help = command().render_help().to_string();
        for heading in [
            "Uso: ",
            "Comandi:",
            "Opzioni:",
            "Mostra l'aiuto",
            "Mostra la versione",
        ] {
            assert!(help.contains(heading), "{heading}");
        }
    }

    #[test]
    fn numbers_too_large_for_the_type_are_explained_in_italian() {
        assert_eq!(
            error_of(&[
                "portolano",
                "add",
                "ssh",
                "/x",
                "--host",
                "h",
                "--port",
                "70000"
            ]),
            "valore non valido '70000' per --port <PORT>: valore fuori intervallo (0..=65535)"
        );
    }

    #[test]
    fn invalid_digits_are_explained_in_italian() {
        assert_eq!(
            error_of(&[
                "portolano",
                "add",
                "ssh",
                "/x",
                "--host",
                "h",
                "--port",
                "abc"
            ]),
            "valore non valido 'abc' per --port <PORT>: cifra non valida"
        );
    }

    #[test]
    fn numbers_out_of_range_are_explained_in_italian() {
        assert_eq!(
            error_of(&[
                "portolano",
                "add",
                "rdp",
                "/x",
                "--host",
                "h",
                "--scale",
                "99999999999"
            ]),
            "valore non valido '99999999999' per --scale <N>: valore fuori intervallo (0..=4294967295)"
        );
    }

    #[test]
    fn empty_numbers_are_explained_in_italian() {
        assert_eq!(
            error_of(&["portolano", "tree", "--depth", ""]),
            "valore non valido '' per --depth <N>: valore vuoto"
        );
    }
}
