//! Terminal colours for the text output.
//!
//! Colour is on when the `--color` flag says so, or, in `auto` mode, when the
//! stream is a terminal and the environment allows it. Machine-readable output
//! is never coloured, and the output of a non-terminal is never coloured in
//! `auto` mode, so pipes and files see the same bytes as before colours existed.

use std::fmt::Display;
use std::io::IsTerminal;

use anstyle::{AnsiColor, Color, Effects, Style};
use clap::ValueEnum;

/// The `--color` flag.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum ColorWhen {
    /// Colour on a terminal, unless the environment says otherwise.
    #[default]
    Auto,
    /// Always colour, whatever the stream or the environment.
    Always,
    /// Never colour.
    Never,
}

/// Whether each text stream is coloured.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Colours {
    /// Diagnostics and the summary (stdout).
    pub stdout: bool,
    /// Warnings, errors and `-v` lines (stderr).
    pub stderr: bool,
}

impl Colours {
    /// The colours for this process: the flag, the environment and whether each
    /// stream is a terminal. Enables the escape codes on Windows when colour is on.
    pub fn for_process(when: ColorWhen) -> Self {
        let env = |key: &str| std::env::var(key).ok();
        let colours = Self::decide(
            when,
            &env,
            std::io::stdout().is_terminal(),
            std::io::stderr().is_terminal(),
        );
        if colours.stdout || colours.stderr {
            // Windows consoles need virtual terminal processing for the codes;
            // this is a no-op elsewhere.
            let _ = anstyle_query::windows::enable_ansi_colors();
        }
        colours
    }

    /// The pure decision, for the given flag, environment lookup and terminal
    /// state of each stream.
    pub fn decide(
        when: ColorWhen,
        env: &dyn Fn(&str) -> Option<String>,
        stdout_terminal: bool,
        stderr_terminal: bool,
    ) -> Self {
        Self {
            stdout: enabled(when, env, stdout_terminal),
            stderr: enabled(when, env, stderr_terminal),
        }
    }
}

/// Whether one stream is coloured.
///
/// `always` and `never` win over everything. In `auto` mode, the first rule that
/// applies decides:
/// 1. `NO_COLOR` is set and not empty: off.
/// 2. `CLICOLOR_FORCE` or `FORCE_COLOR` is set and not `0`: on.
/// 3. `CLICOLOR` is `0`: off.
/// 4. `TERM` is `dumb`: off.
/// 5. Otherwise on exactly when the stream is a terminal.
fn enabled(when: ColorWhen, env: &dyn Fn(&str) -> Option<String>, terminal: bool) -> bool {
    match when {
        ColorWhen::Always => true,
        ColorWhen::Never => false,
        ColorWhen::Auto => {
            let set = |key: &str| env(key).filter(|value| !value.is_empty());
            let forced = |key: &str| set(key).is_some_and(|value| value != "0");
            if set("NO_COLOR").is_some() {
                false
            } else if forced("CLICOLOR_FORCE") || forced("FORCE_COLOR") {
                true
            } else if env("CLICOLOR").is_some_and(|value| value == "0")
                || env("TERM").is_some_and(|value| value == "dumb")
            {
                false
            } else {
                terminal
            }
        }
    }
}

/// Text in `style` when `on`, otherwise the text unchanged.
pub fn paint(on: bool, style: Style, text: impl Display) -> String {
    if on {
        format!("{}{text}{}", style.render(), style.render_reset())
    } else {
        text.to_string()
    }
}

/// The style of an error: `error:` on stderr and the `[E]` tag.
pub fn error() -> Style {
    Style::new()
        .bold()
        .fg_color(Some(Color::Ansi(AnsiColor::Red)))
}

/// The style of a warning: `warning:` on stderr and the `[W]` tag.
pub fn warning() -> Style {
    Style::new().fg_color(Some(Color::Ansi(AnsiColor::Yellow)))
}

/// The style of an info finding: the `[I]` tag.
pub fn info() -> Style {
    Style::new().fg_color(Some(Color::Ansi(AnsiColor::Cyan)))
}

/// The style of the file path and the check ID.
pub fn emphasis() -> Style {
    Style::new().bold()
}

/// The style of the line and column, and of the `malt:` prefix of `-v` lines.
pub fn dim() -> Style {
    Style::new().effects(Effects::DIMMED)
}

/// The style of the hint about fixes and of the success line.
pub fn success() -> Style {
    Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        move |key| map.get(key).cloned()
    }

    fn auto(pairs: &[(&str, &str)], terminal: bool) -> bool {
        enabled(ColorWhen::Auto, &env(pairs), terminal)
    }

    #[test]
    fn flag_always_and_never_win_over_everything() {
        let noisy = env(&[("NO_COLOR", "1"), ("TERM", "dumb")]);
        assert!(enabled(ColorWhen::Always, &noisy, false));
        assert!(!enabled(
            ColorWhen::Never,
            &env(&[("FORCE_COLOR", "1")]),
            true
        ));
    }

    #[test]
    fn auto_follows_the_terminal_without_env() {
        assert!(auto(&[], true));
        assert!(!auto(&[], false));
    }

    #[test]
    fn no_color_disables_unless_empty() {
        assert!(!auto(&[("NO_COLOR", "1")], true));
        assert!(auto(&[("NO_COLOR", "")], true));
    }

    #[test]
    fn no_color_beats_the_forcing_variables() {
        assert!(!auto(&[("NO_COLOR", "1"), ("FORCE_COLOR", "1")], false));
        assert!(!auto(&[("NO_COLOR", "1"), ("CLICOLOR_FORCE", "1")], false));
    }

    #[test]
    fn forcing_variables_enable_without_a_terminal() {
        assert!(auto(&[("CLICOLOR_FORCE", "1")], false));
        assert!(auto(&[("FORCE_COLOR", "1")], false));
        assert!(auto(&[("FORCE_COLOR", "yes")], false));
    }

    #[test]
    fn zero_or_empty_forcing_variables_do_not_force() {
        assert!(!auto(&[("CLICOLOR_FORCE", "0")], false));
        assert!(!auto(&[("FORCE_COLOR", "0")], false));
        assert!(!auto(&[("FORCE_COLOR", "")], false));
        assert!(auto(&[("FORCE_COLOR", "0")], true));
    }

    #[test]
    fn forcing_beats_clicolor_zero_and_term_dumb() {
        assert!(auto(&[("CLICOLOR", "0"), ("CLICOLOR_FORCE", "1")], false));
        assert!(auto(&[("TERM", "dumb"), ("FORCE_COLOR", "1")], false));
    }

    #[test]
    fn clicolor_zero_disables_on_a_terminal() {
        assert!(!auto(&[("CLICOLOR", "0")], true));
        assert!(auto(&[("CLICOLOR", "1")], true));
    }

    #[test]
    fn term_dumb_disables_on_a_terminal() {
        assert!(!auto(&[("TERM", "dumb")], true));
        assert!(auto(&[("TERM", "xterm-256color")], true));
    }

    #[test]
    fn each_stream_follows_its_own_terminal() {
        let colours = Colours::decide(ColorWhen::Auto, &env(&[]), true, false);
        assert_eq!(
            colours,
            Colours {
                stdout: true,
                stderr: false
            }
        );
        let colours = Colours::decide(ColorWhen::Always, &env(&[]), false, false);
        assert!(colours.stdout && colours.stderr);
    }

    #[test]
    fn paint_is_the_identity_when_off() {
        assert_eq!(paint(false, error(), "[E]"), "[E]");
        let on = paint(true, error(), "[E]");
        assert!(on.starts_with("\x1b[") && on.ends_with("\x1b[0m"), "{on:?}");
        assert!(on.contains("[E]"));
    }
}
