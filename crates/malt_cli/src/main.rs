use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use clap::{ArgAction, Parser};

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

    /// Report on stderr which files are linted (`-v`), with the time and the
    /// number of findings of each file (`-vv`). The diagnostics on stdout do
    /// not change.
    #[arg(short, long, action = ArgAction::Count)]
    verbose: u8,
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
    let registry = Arc::new(RuleRegistry::new(rules, &config));
    let cores = std::thread::available_parallelism().map_or(1, |n| n.get());
    let new_linter = || {
        let mut linter = Linter::shared(registry.clone());
        linter.set_inline_suppression(config.inline_suppression);
        // A large file also spreads its own checks over the cores.
        linter.set_file_check_threads(cores, LINT_STACK_BYTES);
        linter
    };

    let started = Instant::now();
    let mut summary = Summary::default();
    let threads = cores.min(cli.files.len());
    if cli.fix || threads <= 1 {
        // `--fix` writes files and reports each fix as it goes, so it works
        // through them one by one, in order.
        let mut linter = new_linter();
        for file_path in &cli.files {
            let checked = check_file(&mut linter, file_path, &cli)?;
            summary.report(file_path, checked, cli.verbose);
        }
    } else {
        lint_in_parallel(&cli, threads, new_linter, |file_path, checked| {
            summary.report(file_path, checked?, cli.verbose);
            Ok(())
        })?;
    }
    if cli.verbose > 0 {
        eprintln!(
            "malt: {} file{} in {}",
            cli.files.len(),
            if cli.files.len() == 1 { "" } else { "s" },
            format_duration(started.elapsed())
        );
    }

    if summary.total_diagnostics > 0 {
        println!(
            "\nFound {} issue{} in {} file{}.",
            summary.total_diagnostics,
            if summary.total_diagnostics == 1 {
                ""
            } else {
                "s"
            },
            summary.files_with_issues,
            if summary.files_with_issues == 1 {
                ""
            } else {
                "s"
            },
        );
        if summary.fixable > 0 {
            println!("{} fixable with the `--fix` option.", summary.fixable);
        }
        if summary.unsafe_fixable > 0 && !cli.unsafe_fixes {
            println!(
                "{} more with `--fix --unsafe-fixes` (these may change behaviour).",
                summary.unsafe_fixable
            );
        }
        process::exit(1);
    }

    Ok(())
}

/// Lint the files of `cli` on `threads` threads, each with its own linter from
/// `new_linter`, and hand each file's outcome to `report` in the order of the
/// command line (so the output does not depend on which thread is faster).
/// Stops at the first error `report` returns.
fn lint_in_parallel(
    cli: &Cli,
    threads: usize,
    new_linter: impl Fn() -> Linter + Sync,
    mut report: impl FnMut(&Path, Result<Checked>) -> Result<()>,
) -> Result<()> {
    let files = &cli.files;
    let next = AtomicUsize::new(0);
    let (sender, receiver) = mpsc::channel();
    std::thread::scope(|scope| {
        let mut started = 0;
        for _ in 0..threads {
            let sender = sender.clone();
            let (next, new_linter) = (&next, &new_linter);
            let worker = std::thread::Builder::new()
                .name("malt".into())
                .stack_size(LINT_STACK_BYTES)
                .spawn_scoped(scope, move || {
                    let mut linter = new_linter();
                    loop {
                        let index = next.fetch_add(1, Ordering::Relaxed);
                        let Some(file_path) = files.get(index) else {
                            break;
                        };
                        let outcome = check_file(&mut linter, file_path, cli);
                        // The receiver is gone once reporting stopped at an error.
                        if sender.send((index, outcome)).is_err() {
                            break;
                        }
                    }
                });
            if worker.is_ok() {
                started += 1;
            }
        }
        drop(sender);
        if started == 0 {
            anyhow::bail!("could not start the analyzer threads");
        }
        // Outcomes arrive in any order: hold them until their turn.
        let mut waiting = BTreeMap::new();
        let mut turn = 0;
        for (index, outcome) in receiver {
            waiting.insert(index, outcome);
            while let Some(outcome) = waiting.remove(&turn) {
                report(&files[turn], outcome)?;
                turn += 1;
            }
        }
        Ok(())
    })
}

/// What reading, linting and (with `--fix`) fixing one file gave.
enum FileOutcome {
    /// The file could not be read: its NOFIL or RDERR diagnostic.
    Unreadable(Diagnostic),
    /// The diagnostics that remain.
    Linted(Vec<Diagnostic>),
}

/// A file's outcome with the time it took to read, lint and fix it.
struct Checked {
    outcome: FileOutcome,
    elapsed: Duration,
}

/// Read, lint and, with `--fix`, fix one file, and time it.
fn check_file(linter: &mut Linter, file_path: &Path, cli: &Cli) -> Result<Checked> {
    let started = Instant::now();
    let outcome = check_file_untimed(linter, file_path, cli)?;
    Ok(Checked {
        outcome,
        elapsed: started.elapsed(),
    })
}

fn check_file_untimed(linter: &mut Linter, file_path: &Path, cli: &Cli) -> Result<FileOutcome> {
    let source = match fs::read_to_string(file_path) {
        Ok(s) => s,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            // NOFIL — file not found. Skip lint + fix; never write an empty file
            // to a missing path.
            return Ok(FileOutcome::Unreadable(Diagnostic {
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
            }));
        }
        Err(_) => {
            // RDERR — non-NotFound I/O error (permission denied, directory, etc.).
            // Catch-all Err(_) is intentional: std::io::ErrorKind has no stable
            // IsADirectory variant.
            return Ok(FileOutcome::Unreadable(Diagnostic {
                rule_id: "RDERR",
                message: format!("Unable to read file {}.", file_path.display()),
                severity: Severity::Error,
                byte_range: 0..1,
                line: 1,
                column: 1,
                fix: None,
            }));
        }
    };

    let diagnostics = if cli.fix {
        let mode = if cli.unsafe_fixes {
            FixMode::All
        } else {
            FixMode::Safe
        };
        let outcome = fix_source(linter, &source, file_path, mode)
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
    Ok(FileOutcome::Linted(diagnostics))
}

/// The counts behind the closing summary line.
#[derive(Default)]
struct Summary {
    total_diagnostics: usize,
    files_with_issues: usize,
    fixable: usize,
    unsafe_fixable: usize,
}

impl Summary {
    /// Print the diagnostics of one file and count them.
    fn report(&mut self, file_path: &Path, checked: Checked, verbose: u8) {
        let Checked { outcome, elapsed } = checked;
        let diagnostics = match outcome {
            FileOutcome::Unreadable(diag) => {
                if verbose > 0 {
                    eprintln!("malt: skipped {} ({})", file_path.display(), diag.rule_id);
                }
                print_diagnostic(file_path, &diag);
                self.total_diagnostics += 1;
                self.files_with_issues += 1;
                return;
            }
            FileOutcome::Linted(diagnostics) => diagnostics,
        };
        match verbose {
            0 => {}
            1 => eprintln!("malt: linted {}", file_path.display()),
            _ => eprintln!(
                "malt: linted {} in {} ({} finding{})",
                file_path.display(),
                format_duration(elapsed),
                diagnostics.len(),
                if diagnostics.len() == 1 { "" } else { "s" }
            ),
        }
        if diagnostics.is_empty() {
            return;
        }
        for diag in &diagnostics {
            print_diagnostic(file_path, diag);
            match diag.fix.as_ref() {
                Some(f) if f.is_safe() => self.fixable += 1,
                Some(_) => self.unsafe_fixable += 1,
                None => {}
            }
        }
        self.total_diagnostics += diagnostics.len();
        self.files_with_issues += 1;
    }
}

/// A duration for humans: microseconds below a millisecond, milliseconds
/// below a second, seconds above.
fn format_duration(duration: Duration) -> String {
    let micros = duration.as_micros();
    if micros < 1_000 {
        format!("{micros} µs")
    } else if micros < 1_000_000 {
        format!("{:.2} ms", micros as f64 / 1_000.0)
    } else {
        format!("{:.2} s", micros as f64 / 1_000_000.0)
    }
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
