//! The help in Italian (DEC-023.2): clap writes its headings, value lists
//! and defaults in English, so the command is adjusted before parsing.

use clap::builder::PossibleValue;
use clap::{Arg, ArgAction, Command, CommandFactory};

use super::Cli;

/// Heading of the subcommands in the help.
const COMMANDS_HEADING: &str = "Comandi";
/// Heading of the positional arguments.
const ARGUMENTS_HEADING: &str = "Argomenti";
/// Heading of the options of the command itself.
const OPTIONS_HEADING: &str = "Opzioni";
/// Heading of the options valid for every command (SPEC §10.1).
const GLOBAL_OPTIONS_HEADING: &str = "Opzioni globali";

/// Help layout: `{all-args}` writes one section per heading. Every argument
/// gets an Italian heading (see [`localize_arg`]), since clap's default
/// headings are in English and cannot be translated otherwise.
const HELP_TEMPLATE: &str = "{about-with-newline}\nUso: {usage}\n\n{all-args}{after-help}";

/// The command of [`Cli`] with Italian headings, value lists and help
/// flags.
pub fn command() -> Command {
    let version = Arg::new("version")
        .short('V')
        .long("version")
        .action(ArgAction::Version)
        .help("Mostra la versione")
        .help_heading(OPTIONS_HEADING);
    localize(Cli::command())
        .disable_version_flag(true)
        .arg(version)
}

fn localize(command: Command) -> Command {
    let help = Arg::new("help")
        .short('h')
        .long("help")
        .action(ArgAction::Help)
        .help("Mostra l'aiuto");
    command
        .disable_help_flag(true)
        .disable_help_subcommand(true)
        .arg(help)
        .subcommand_help_heading(COMMANDS_HEADING)
        .mut_args(localize_arg)
        .help_template(HELP_TEMPLATE)
        .mut_subcommands(localize)
}

/// The Italian heading of `arg` and, for an option with a list of values or
/// a default, those in Italian. clap propagates the global options to the
/// subcommands with their heading, so they appear there under
/// [`GLOBAL_OPTIONS_HEADING`].
fn localize_arg(arg: Arg) -> Arg {
    let heading = if arg.is_positional() {
        ARGUMENTS_HEADING
    } else if arg.is_global_set() {
        GLOBAL_OPTIONS_HEADING
    } else {
        OPTIONS_HEADING
    };
    with_italian_values(arg.help_heading(heading))
}

/// clap writes the accepted values as `[possible values: …]` (under
/// `Possible values:` in the long help) and the default as `[default: …]`,
/// with no way to translate them: they are hidden and written in Italian
/// into the help text instead.
fn with_italian_values(mut arg: Arg) -> Arg {
    // Before the command is built, flags also report the values of their
    // parser (`true`, `false`): only options that take a value have them.
    if !arg.get_action().takes_values() {
        return arg;
    }
    // Notes appended to the short help, and paragraphs to the long one.
    let mut short = Vec::new();
    let mut long = Vec::new();
    let values: Vec<PossibleValue> = arg
        .get_possible_values()
        .into_iter()
        .filter(|value| !value.is_hide_set())
        .collect();
    if !values.is_empty() && !arg.is_hide_possible_values_set() {
        let names: Vec<&str> = values.iter().map(PossibleValue::get_name).collect();
        let note = format!("[valori ammessi: {}]", names.join(", "));
        if values.iter().any(|value| value.get_help().is_some()) {
            let lines: Vec<String> = values
                .iter()
                .map(|value| match value.get_help() {
                    Some(text) => format!("- {}: {text}", value.get_name()),
                    None => format!("- {}", value.get_name()),
                })
                .collect();
            long.push(format!("Valori ammessi:\n{}", lines.join("\n")));
        } else {
            long.push(note.clone());
        }
        short.push(note);
        arg = arg.hide_possible_values(true);
    }
    let defaults: Vec<String> = arg
        .get_default_values()
        .iter()
        .map(|value| value.to_string_lossy().into_owned())
        .collect();
    if !defaults.is_empty() && !arg.is_hide_default_value_set() {
        let note = format!("[predefinito: {}]", defaults.join(" "));
        short.push(note.clone());
        long.push(note);
        arg = arg.hide_default_value(true);
    }
    if short.is_empty() {
        return arg;
    }
    let help = arg.get_help().map(ToString::to_string).unwrap_or_default();
    let short_help = format!("{help} {}", short.join(" "));
    let long_help = format!("{help}\n\n{}", long.join("\n\n"));
    arg.help(short_help).long_help(long_help)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn subcommand_help(name: &str, long: bool) -> String {
        let mut command = command();
        command.build();
        let subcommand = command.find_subcommand_mut(name).unwrap();
        if long {
            subcommand.render_long_help().to_string()
        } else {
            subcommand.render_help().to_string()
        }
    }

    #[test]
    fn localized_command_is_consistent() {
        command().debug_assert();
    }

    #[test]
    fn help_has_italian_headings() {
        let help = command().render_help().to_string();
        for heading in [
            "Uso: ",
            "Comandi:",
            "Opzioni:",
            "Opzioni globali:",
            "Mostra l'aiuto",
            "Mostra la versione",
        ] {
            assert!(help.contains(heading), "{heading}");
        }
    }

    #[test]
    fn global_options_have_their_own_heading_after_the_command_options() {
        for long in [false, true] {
            let help = subcommand_help("tree", long);

            let options = help.find("\nOpzioni:\n").unwrap();
            let globals = help.find("\nOpzioni globali:\n").unwrap();
            let file = help.find("--file").unwrap();
            let depth = help.find("--depth").unwrap();
            assert!(
                options < depth && depth < globals && globals < file,
                "{help}"
            );
            assert!(help.contains("\nArgomenti:\n"), "{help}");
        }
    }

    #[test]
    fn accepted_values_and_defaults_are_listed_in_italian() {
        for long in [false, true] {
            let help = subcommand_help("import", long);

            assert!(help.contains("alori ammessi"), "{help}");
            assert!(help.contains("[predefinito: fail]"), "{help}");
            assert!(!help.contains("ossible values"), "{help}");
            assert!(!help.contains("[default:"), "{help}");
        }
    }

    #[test]
    fn long_help_describes_each_accepted_value() {
        let help = subcommand_help("import", true);

        assert!(help.contains("Valori ammessi:\n"), "{help}");
        assert!(help.contains("- new-ids: Importa con id nuovi"), "{help}");
    }

    #[test]
    fn flags_list_no_values() {
        let help = subcommand_help("delete", true);

        assert!(!help.contains("valori ammessi"), "{help}");
    }
}
