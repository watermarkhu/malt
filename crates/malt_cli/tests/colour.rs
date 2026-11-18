//! `--color` and the environment decide whether the text output has terminal
//! colours. The tests run the binary with stdout and stderr on pipes, so
//! `auto` has no terminal to find: only the flag and the forcing variables
//! can turn colour on there.

use std::path::Path;
use std::process::{Command, Output};

const ENV_KEYS: [&str; 5] = [
    "NO_COLOR",
    "CLICOLOR",
    "CLICOLOR_FORCE",
    "FORCE_COLOR",
    "TERM",
];

/// A file with a warning (FNDEF) and an info finding (NOPRT).
const FINDINGS: &str = "function r = b(x)\nr = x\nend\n";

fn malt(dir: &Path, args: &[&str], env: &[(&str, &str)]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_malt"));
    command.current_dir(dir).args(args);
    for key in ENV_KEYS {
        command.env_remove(key);
    }
    for (key, value) in env {
        command.env(key, value);
    }
    command.output().expect("failed to run malt")
}

fn project() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    std::fs::write(dir.path().join("a.m"), FINDINGS).expect("failed to write test file");
    let file = std::path::PathBuf::from("a.m");
    (dir, file)
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// `text` without its SGR escape sequences (`ESC [ ... m`).
fn strip(text: &str) -> String {
    let mut plain = String::new();
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            for next in chars.by_ref() {
                if next == 'm' {
                    break;
                }
            }
        } else {
            plain.push(c);
        }
    }
    plain
}

fn has_escapes(text: &str) -> bool {
    text.contains('\x1b')
}

#[test]
fn always_colours_the_severity_tags_and_the_summary() {
    let (dir, file) = project();
    let coloured = text(
        &malt(
            dir.path(),
            &["--color", "always", file.to_str().unwrap()],
            &[],
        )
        .stdout,
    );
    assert!(coloured.contains("\x1b[33m[W]\x1b[0m"), "{coloured:?}");
    assert!(coloured.contains("\x1b[36m[I]\x1b[0m"), "{coloured:?}");
    assert!(
        coloured.contains("\x1b[1mFound 2 issues in 1 file.\x1b[0m"),
        "{coloured:?}"
    );
    assert!(coloured.contains("\x1b[1mFNDEF\x1b[0m"), "{coloured:?}");
    assert!(coloured.contains("\x1b[2m:1:14\x1b[0m"), "{coloured:?}");
}

#[test]
fn always_strips_back_to_the_plain_output() {
    let (dir, file) = project();
    let args = [file.to_str().unwrap()];
    let plain = text(&malt(dir.path(), &["--color", "never", args[0]], &[]).stdout);
    for flag in ["always", "auto"] {
        let coloured = text(
            &malt(
                dir.path(),
                &["--color", flag, args[0]],
                &[("FORCE_COLOR", "1")],
            )
            .stdout,
        );
        assert!(has_escapes(&coloured), "{flag}: {coloured:?}");
        assert_eq!(strip(&coloured), plain, "{flag}");
    }
}

#[test]
fn never_and_piped_auto_have_no_escape_bytes() {
    let (dir, file) = project();
    let never = malt(
        dir.path(),
        &["--color", "never", file.to_str().unwrap()],
        &[],
    );
    let auto = malt(dir.path(), &[file.to_str().unwrap()], &[]);
    assert!(!has_escapes(&text(&never.stdout)));
    assert!(!has_escapes(&text(&auto.stdout)));
    assert_eq!(never.stdout, auto.stdout);
}

#[test]
fn colour_spelling_is_an_alias() {
    let (dir, file) = project();
    let a = malt(
        dir.path(),
        &["--colour", "always", file.to_str().unwrap()],
        &[],
    );
    let b = malt(
        dir.path(),
        &["--color", "always", file.to_str().unwrap()],
        &[],
    );
    assert_eq!(a.stdout, b.stdout);
}

#[test]
fn environment_precedence_in_auto_mode() {
    let (dir, file) = project();
    let path = file.to_str().unwrap();
    let table: [(&[(&str, &str)], bool); 12] = [
        (&[], false),
        (&[("FORCE_COLOR", "1")], true),
        (&[("CLICOLOR_FORCE", "1")], true),
        (&[("FORCE_COLOR", "0")], false),
        (&[("CLICOLOR_FORCE", "0")], false),
        (&[("FORCE_COLOR", "")], false),
        (&[("NO_COLOR", "1"), ("FORCE_COLOR", "1")], false),
        (&[("NO_COLOR", "1"), ("CLICOLOR_FORCE", "1")], false),
        (&[("NO_COLOR", ""), ("FORCE_COLOR", "1")], true),
        (&[("CLICOLOR", "0"), ("CLICOLOR_FORCE", "1")], true),
        (&[("TERM", "dumb"), ("FORCE_COLOR", "1")], true),
        (&[("TERM", "dumb"), ("CLICOLOR", "1")], false),
    ];
    for (env, coloured) in table {
        let output = malt(dir.path(), &[path], env);
        assert_eq!(has_escapes(&text(&output.stdout)), coloured, "env {env:?}");
    }
}

#[test]
fn flag_overrides_the_environment() {
    let (dir, file) = project();
    let path = file.to_str().unwrap();
    let never = malt(
        dir.path(),
        &["--color", "never", path],
        &[("FORCE_COLOR", "1")],
    );
    assert!(!has_escapes(&text(&never.stdout)));
    let always = malt(
        dir.path(),
        &["--color", "always", path],
        &[("NO_COLOR", "1")],
    );
    assert!(has_escapes(&text(&always.stdout)));
}

#[test]
fn stderr_errors_are_red_and_follow_the_same_rules() {
    let (dir, _file) = project();
    let args = ["--config", "missing.toml", "a.m"];
    let with = |flag: &'static str| vec!["--color", flag, args[0], args[1], args[2]];
    let coloured = malt(dir.path(), &with("always"), &[]);
    let stderr = text(&coloured.stderr);
    assert!(
        stderr.starts_with("\x1b[1m\x1b[31merror\x1b[0m: "),
        "{stderr:?}"
    );
    let plain = malt(dir.path(), &with("never"), &[]);
    assert_eq!(
        text(&plain.stderr),
        "error: config file not found: missing.toml\n"
    );
    assert_eq!(strip(&stderr), text(&plain.stderr));
    let forced = malt(dir.path(), &args, &[("FORCE_COLOR", "1")]);
    assert!(has_escapes(&text(&forced.stderr)));
    let piped = malt(dir.path(), &args, &[]);
    assert!(!has_escapes(&text(&piped.stderr)));
}

#[test]
fn stderr_warnings_are_yellow() {
    let (dir, file) = project();
    std::fs::write(
        dir.path().join(".malt.toml"),
        "[lint.rules]\nNOSMI = \"off\"\n",
    )
    .expect("failed to write config");
    let path = file.to_str().unwrap();
    let coloured = text(&malt(dir.path(), &["--color", "always", path], &[]).stderr);
    assert!(coloured.contains("\x1b[33mwarning:\x1b[0m"), "{coloured:?}");
    let plain = text(&malt(dir.path(), &["--color", "never", path], &[]).stderr);
    assert_eq!(strip(&coloured), plain);
    assert!(plain.starts_with("warning: .malt.toml: "), "{plain:?}");
}

#[test]
fn verbose_lines_dim_their_prefix_on_stderr_only() {
    let (dir, file) = project();
    let path = file.to_str().unwrap();
    let coloured = malt(dir.path(), &["-v", "--color", "always", path], &[]);
    let stderr = text(&coloured.stderr);
    assert!(
        stderr.contains("\x1b[2mmalt:\x1b[0m linted a.m"),
        "{stderr:?}"
    );
    let plain = malt(dir.path(), &["-v", "--color", "never", path], &[]);
    let plain_lines: Vec<String> = text(&plain.stderr).lines().map(str::to_owned).collect();
    let stripped: Vec<String> = strip(&stderr).lines().map(str::to_owned).collect();
    assert_eq!(stripped.len(), plain_lines.len());
    assert_eq!(stripped[0], plain_lines[0]);
    assert_eq!(strip(&text(&coloured.stdout)), text(&plain.stdout));
    assert!(has_escapes(&text(&coloured.stdout)));
}
