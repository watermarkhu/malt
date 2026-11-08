use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;

use anyhow::{Context, Result};
use clap::Parser;

use malt_core::{fix_source, Config, Diagnostic, FixMode, Linter, RuleRegistry, Severity};

/// malt — An ultra-fast, extensible linter for MATLAB.
#[derive(Parser, Debug)]
#[command(name = "malt", version, about)]
struct Cli {
    /// MATLAB files to lint.
    #[arg(required = true)]
    files: Vec<PathBuf>,

    /// Apply safe auto-fixes where available, repeating until nothing changes.
    #[arg(long)]
    fix: bool,

    /// Also apply fixes that may change behaviour (requires --fix).
    #[arg(long, requires = "fix")]
    unsafe_fixes: bool,

    /// Path to config file (default: .malt.toml in current directory).
    #[arg(long, value_name = "PATH")]
    config: Option<PathBuf>,

    /// Named preset to base the configuration on (all, mathworks, recommended).
    /// Overrides the `[lint] preset` value in the config file.
    #[arg(long, value_name = "PRESET")]
    preset: Option<String>,
}

/// Stack size of the thread that lints. Many rules walk the syntax tree
/// recursively, and the 8 MiB of a main thread (2 MiB on some platforms) is
/// too little for deeply nested code. The memory is only reserved, and only
/// the part that is used gets committed.
const LINT_STACK_BYTES: usize = 256 * 1024 * 1024;

fn main() {
    let cli = Cli::parse();

    let worker = std::thread::Builder::new()
        .name("malt".into())
        .stack_size(LINT_STACK_BYTES)
        .spawn(move || run(cli));
    let result = match worker {
        Ok(handle) => handle
            .join()
            .unwrap_or_else(|_| Err(anyhow::anyhow!("the analyzer thread panicked"))),
        // Could not reserve the stack: fall back to the thread we are on.
        Err(_) => Err(anyhow::anyhow!("could not start the analyzer thread")),
    };
    if let Err(err) = result {
        eprintln!("error: {err:#}");
        process::exit(2);
    }
}

fn run(cli: Cli) -> Result<()> {
    // Load configuration from .malt.toml (or --config path).
    let mut config = load_config(cli.config.as_deref())?;

    // A --preset override takes priority over the config file's `[lint] preset`.
    if let Some(name) = &cli.preset {
        config.preset = malt_core::preset::resolve(name).ok_or_else(|| {
            let names = malt_core::preset::PRESETS
                .iter()
                .map(|p| p.name)
                .collect::<Vec<_>>()
                .join(", ");
            anyhow::anyhow!("unknown preset '{name}' (expected one of: {names})")
        })?;
    }

    // Build rule registry from configuration (only enabled rules).
    let rules = malt_rules::active_rules(&config);
    let registry = RuleRegistry::new(rules, &config);
    let mut linter = Linter::new(registry);
    linter.set_inline_suppression(config.inline_suppression);

    let mut total_diagnostics = 0;
    let mut files_with_issues = 0;
    let mut fixable = 0;
    let mut unsafe_fixable = 0;

    for file_path in &cli.files {
        let source = match fs::read_to_string(file_path) {
            Ok(s) => s,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                // NOFIL — file not found.
                let diag = Diagnostic {
                    rule_id: "NOFIL",
                    message: format!(
                        "Unable to open file {}. File is not found.",
                        file_path.display()
                    ),
                    severity: Severity::Error,
                    byte_range: 0..1,
                    line: 1,
                    column: 1,
                    fix: None,
                };
                print_diagnostic(file_path, &diag);
                total_diagnostics += 1;
                files_with_issues += 1;
                continue; // skip lint + fix; never write an empty file to a missing path
            }
            Err(_) => {
                // RDERR — non-NotFound I/O error (permission denied, directory, etc.).
                // Catch-all Err(_) is intentional: std::io::ErrorKind has no stable
                // IsADirectory variant.
                let diag = Diagnostic {
                    rule_id: "RDERR",
                    message: format!("Unable to read file {}.", file_path.display()),
                    severity: Severity::Error,
                    byte_range: 0..1,
                    line: 1,
                    column: 1,
                    fix: None,
                };
                print_diagnostic(file_path, &diag);
                total_diagnostics += 1;
                files_with_issues += 1;
                continue;
            }
        };

        let diagnostics = if cli.fix {
            let mode = if cli.unsafe_fixes {
                FixMode::All
            } else {
                FixMode::Safe
            };
            let outcome = fix_source(&mut linter, &source, file_path, mode)
                .with_context(|| format!("failed to parse {}", file_path.display()))?;
            // Only touch the file when something was actually fixed.
            if outcome.source != source {
                fs::write(file_path, &outcome.source)
                    .with_context(|| format!("failed to write {}", file_path.display()))?;
                println!(
                    "Fixed {} issue{} in {}",
                    outcome.applied,
                    if outcome.applied == 1 { "" } else { "s" },
                    file_path.display()
                );
            }
            outcome.remaining
        } else {
            linter
                .lint(&source, file_path)
                .with_context(|| format!("failed to parse {}", file_path.display()))?
        };

        if diagnostics.is_empty() {
            continue;
        }

        for diag in &diagnostics {
            print_diagnostic(file_path, diag);
            match diag.fix.as_ref() {
                Some(f) if f.is_safe() => fixable += 1,
                Some(_) => unsafe_fixable += 1,
                None => {}
            }
        }

        total_diagnostics += diagnostics.len();
        files_with_issues += 1;
    }

    if total_diagnostics > 0 {
        println!(
            "\nFound {} issue{} in {} file{}.",
            total_diagnostics,
            if total_diagnostics == 1 { "" } else { "s" },
            files_with_issues,
            if files_with_issues == 1 { "" } else { "s" },
        );
        if fixable > 0 {
            println!("{fixable} fixable with the `--fix` option.");
        }
        if unsafe_fixable > 0 && !cli.unsafe_fixes {
            println!(
                "{unsafe_fixable} more with `--fix --unsafe-fixes` (these may change behaviour)."
            );
        }
        process::exit(1);
    }

    Ok(())
}

/// Print a diagnostic in the standard `file:line:col [E] ID: message` format.
fn print_diagnostic(file_path: &Path, diag: &Diagnostic) {
    let severity = match diag.severity {
        Severity::Error => "E",
        Severity::Warning => "W",
        Severity::Info => "I",
    };
    println!(
        "{}:{}:{} [{}] {}: {}",
        file_path.display(),
        diag.line,
        diag.column,
        severity,
        diag.rule_id,
        diag.message,
    );
}

/// Discover and load the configuration file.
///
/// Priority:
/// 1. Explicit `--config` path (error if not found).
/// 2. `.malt.toml` in the current working directory (silent default if missing).
fn load_config(explicit_path: Option<&std::path::Path>) -> Result<Config> {
    let config_path = if let Some(path) = explicit_path {
        if !path.exists() {
            anyhow::bail!("config file not found: {}", path.display());
        }
        Some(path.to_path_buf())
    } else {
        let default_path = PathBuf::from(".malt.toml");
        if default_path.exists() {
            Some(default_path)
        } else {
            None
        }
    };

    match config_path {
        Some(path) => {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("failed to read config: {}", path.display()))?;
            Config::from_toml(&content)
                .with_context(|| format!("failed to parse config: {}", path.display()))
        }
        None => Ok(Config::default()),
    }
}
