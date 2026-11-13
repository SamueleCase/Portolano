//! Opening a command in a new terminal window, as configured in `[terminal]`
//! (SPEC §7.2, "Nuova finestra", DEC-004).

use std::path::PathBuf;

use portolano_core::config::{TerminalApp, TerminalConfig};

use crate::ConnectError;
use crate::env::Os;
use crate::plan::LaunchStep;
use crate::protocol::PlanContext;

/// Placeholder of `terminal.command` replaced by the quoted command.
const CMD_PLACEHOLDER: &str = "{cmd}";

/// The step that relaunches Portolano on the same connection in a new
/// window: `<self_exe> <relaunch_args…> connect <connect_options…> @<id>`.
/// `connect_options` are the options of `connect` that the protocol needs
/// to make the same plan there, such as the client it chose. The command
/// holds no secret: the new process opens the vault itself.
pub(crate) fn relaunch_step(ctx: &PlanContext, connect_options: &[&str]) -> LaunchStep {
    let mut argv = vec![ctx.env.self_exe.to_string_lossy().into_owned()];
    argv.extend(ctx.options.relaunch_args.iter().cloned());
    argv.push("connect".to_owned());
    argv.extend(connect_options.iter().map(|option| (*option).to_owned()));
    argv.push(format!("@{}", ctx.node_id));
    LaunchStep::OpenInTerminal { argv }
}

/// A program and its arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Launcher {
    /// The program.
    pub(crate) program: PathBuf,
    /// Its arguments.
    pub(crate) args: Vec<String>,
}

/// The command that opens a new window of the configured terminal running
/// `argv` (table of SPEC §7.2).
///
/// # Errors
/// [`ConnectError::InvalidConfig`] for a custom terminal without a usable
/// `command`, [`ConnectError::UnsupportedPlatform`] outside macOS and Linux.
pub(crate) fn launcher(
    os: Os,
    terminal: &TerminalConfig,
    argv: &[String],
) -> Result<Launcher, ConnectError> {
    let cmd = shell_words::join(argv);
    match (os, terminal.app) {
        (_, TerminalApp::Custom) => custom_launcher(terminal.command.as_deref(), &cmd),
        (Os::MacOs, TerminalApp::Terminal) => Ok(applescript_launcher(
            "Terminal",
            &format!("do script {}", applescript_string(&cmd)),
        )),
        (Os::MacOs, TerminalApp::Iterm) => Ok(applescript_launcher(
            "iTerm",
            &format!(
                "create window with default profile command {}",
                applescript_string(&cmd)
            ),
        )),
        (Os::MacOs, TerminalApp::Ghostty) => Ok(argv_launcher(
            "open",
            &["-na", "Ghostty", "--args", "-e"],
            argv,
        )),
        (Os::Linux, _) => Ok(argv_launcher("x-terminal-emulator", &["-e"], argv)),
        _ => Err(ConnectError::UnsupportedPlatform {
            feature: "l'apertura in una nuova finestra".to_owned(),
        }),
    }
}

/// `osascript` telling `app` to run `action`, then to come to the front.
fn applescript_launcher(app: &str, action: &str) -> Launcher {
    Launcher {
        program: PathBuf::from("osascript"),
        args: vec![
            "-e".to_owned(),
            format!("tell application \"{app}\" to {action}"),
            "-e".to_owned(),
            format!("tell application \"{app}\" to activate"),
        ],
    }
}

/// `program` with `options` followed by `argv`, unquoted.
fn argv_launcher(program: &str, options: &[&str], argv: &[String]) -> Launcher {
    let mut args: Vec<String> = options.iter().map(|option| (*option).to_owned()).collect();
    args.extend(argv.iter().cloned());
    Launcher {
        program: PathBuf::from(program),
        args,
    }
}

fn custom_launcher(template: Option<&str>, cmd: &str) -> Result<Launcher, ConnectError> {
    let invalid = |reason: &str| ConnectError::InvalidConfig {
        section: "terminal".to_owned(),
        reason: reason.to_owned(),
    };
    let template = template
        .filter(|template| !template.trim().is_empty())
        .ok_or_else(|| invalid("con app = \"custom\" serve `command`"))?;
    if !template.contains(CMD_PLACEHOLDER) {
        return Err(invalid("`command` deve contenere il segnaposto {cmd}"));
    }
    Ok(Launcher {
        program: PathBuf::from("/bin/sh"),
        args: vec!["-c".to_owned(), template.replace(CMD_PLACEHOLDER, cmd)],
    })
}

/// `text` as an AppleScript string literal.
fn applescript_string(text: &str) -> String {
    format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use super::*;
    use crate::protocol::ConnectOptions;
    use crate::testutil::{document, environment};

    fn argv() -> Vec<String> {
        [
            "/Applications/My Apps/portolano",
            "--file",
            "/tmp/it's \"doc\".json",
            "connect",
            "@abc",
        ]
        .map(str::to_owned)
        .to_vec()
    }

    fn config(app: TerminalApp, command: Option<&str>) -> TerminalConfig {
        TerminalConfig {
            app,
            command: command.map(str::to_owned),
        }
    }

    #[test]
    fn relaunch_step_passes_the_global_options_before_connect_and_its_own_after() {
        let doc = document(vec![]);
        let env = environment();
        let app_config = portolano_core::AppConfig::default();
        let node_id = Uuid::new_v4();
        let options = ConnectOptions {
            new_window: true,
            relaunch_args: vec!["--file".into(), "/tmp/doc.json".into()],
            ..ConnectOptions::default()
        };
        let ctx = PlanContext {
            doc: &doc,
            node_id,
            config: &app_config,
            env: &env,
            options: &options,
        };

        let step = relaunch_step(&ctx, &["--wait"]);

        assert_eq!(
            step,
            LaunchStep::OpenInTerminal {
                argv: vec![
                    "/usr/local/bin/portolano".into(),
                    "--file".into(),
                    "/tmp/doc.json".into(),
                    "connect".into(),
                    "--wait".into(),
                    format!("@{node_id}"),
                ],
            }
        );
    }

    #[test]
    fn terminal_app_gets_a_quoted_applescript_command() {
        let launcher = launcher(Os::MacOs, &config(TerminalApp::Terminal, None), &argv()).unwrap();

        assert_eq!(launcher.program, PathBuf::from("osascript"));
        assert_eq!(
            launcher.args[1],
            r#"tell application "Terminal" to do script "'/Applications/My Apps/portolano' --file '/tmp/it'\\''s \"doc\".json' connect @abc""#
        );
        assert_eq!(
            launcher.args[3],
            "tell application \"Terminal\" to activate"
        );
    }

    #[test]
    fn applescript_escapes_backslashes_and_quotes() {
        assert_eq!(applescript_string(r#"a\b"c"#), r#""a\\b\"c""#);
    }

    #[test]
    fn iterm_creates_a_window_with_the_command() {
        let launcher = launcher(Os::MacOs, &config(TerminalApp::Iterm, None), &argv()).unwrap();

        assert!(launcher.args[1].starts_with(
            "tell application \"iTerm\" to create window with default profile command \""
        ));
    }

    #[test]
    fn ghostty_gets_the_argv_unquoted() {
        let launcher = launcher(Os::MacOs, &config(TerminalApp::Ghostty, None), &argv()).unwrap();

        assert_eq!(launcher.program, PathBuf::from("open"));
        assert_eq!(launcher.args[..4], ["-na", "Ghostty", "--args", "-e"]);
        assert_eq!(launcher.args[4..], argv());
    }

    #[test]
    fn custom_command_replaces_the_placeholder() {
        let terminal = config(TerminalApp::Custom, Some("kitty --hold {cmd}"));

        let launcher = launcher(
            Os::MacOs,
            &terminal,
            &["portolano".to_owned(), "connect".to_owned()],
        )
        .unwrap();

        assert_eq!(launcher.program, PathBuf::from("/bin/sh"));
        assert_eq!(launcher.args, ["-c", "kitty --hold portolano connect"]);
    }

    #[test]
    fn custom_command_without_placeholder_is_rejected() {
        let terminal = config(TerminalApp::Custom, Some("kitty"));

        let result = launcher(Os::MacOs, &terminal, &argv());

        assert!(
            matches!(result, Err(ConnectError::InvalidConfig { .. })),
            "{result:?}"
        );
    }

    #[test]
    fn linux_uses_x_terminal_emulator() {
        let launcher = launcher(Os::Linux, &config(TerminalApp::Terminal, None), &argv()).unwrap();

        assert_eq!(launcher.program, PathBuf::from("x-terminal-emulator"));
        assert_eq!(launcher.args[0], "-e");
    }

    #[test]
    fn windows_is_not_supported() {
        let result = launcher(Os::Windows, &config(TerminalApp::Terminal, None), &argv());

        assert!(
            matches!(result, Err(ConnectError::UnsupportedPlatform { .. })),
            "{result:?}"
        );
    }
}
