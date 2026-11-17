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

use malt_core::{
    fix_source, Config, Diagnostic, FixMode, Linter, ProjectIndex, RuleRegistry, Severity,
};

mod colour;
mod ignored;
mod path_index;

use ignored::IgnoredFiles;

use colour::{paint, ColorWhen, Colours};

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

    /// Path to a config file: a `.malt.toml`, or a MATLAB `codeAnalyzerConfiguration.json`
    /// (by extension). Default: `.malt.toml` in the current directory, else
    /// `resources/codeAnalyzerConfiguration.json` there. See the docs.
    #[arg(long, value_name = "PATH")]
    config: Option<PathBuf>,

    /// Named preset to base the configuration on (all, mathworks, recommended).
    /// Overrides the `[lint] preset` value in the config file.
    #[arg(long, value_name = "PRESET")]
    preset: Option<String>,

    /// Exit with an error (code 2) when the config file has a problem: an unknown
    /// rule ID or parameter. Without it, each problem is a warning on stderr.
    #[arg(long)]
    strict_config: bool,

    /// Report on stderr which files are linted (`-v`), with the time and the
    /// number of findings of each file (`-vv`). The diagnostics on stdout do
    /// not change.
    #[arg(short, long, action = ArgAction::Count)]
    verbose: u8,

    /// Use the path index: the classes on the MATLAB search path (`[index]` of the
    /// config file) let some checks report more. Off by default (see the docs).
    #[arg(long, conflicts_with = "no_index")]
    index: bool,

    /// Do not use the path index, even when `[index] enabled = true`.
    #[arg(long, conflicts_with = "index")]
    no_index: bool,

    /// When to colour the text output: `auto` (a terminal, unless the environment
    /// says otherwise), `always` or `never`. See the docs for the environment.
    #[arg(long, alias = "colour", value_name = "WHEN", value_enum, default_value_t = ColorWhen::Auto)]
    color: ColorWhen,

    /// The least severe findings to show: `error` (errors only), `warning` (errors and
    /// warnings) or `info` (everything, the default). Hidden findings are not counted and
    /// do not change the exit code; `--fix` still fixes them.
    #[arg(long, value_name = "LEVEL", value_enum, default_value_t = Level::Info)]
    level: Level,
}

/// How severe a finding must be to be shown (`--level`): that severity or a more severe one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
enum Level {
    /// Errors only.
    Error,
    /// Errors and warnings.
    #[value(alias = "warn")]
    Warning,
    /// Errors, warnings and info (everything).
    Info,
}

impl Level {
    /// Whether a finding of `severity` is shown at this level.
    fn shows(self, severity: Severity) -> bool {
        let rank = match severity {
            Severity::Error => Level::Error,
            Severity::Warning => Level::Warning,
            Severity::Info => Level::Info,
        };
        rank <= self
    }
}

/// Stack size of the thread that lints. Many rules walk the syntax tree
/// recursively, and the 8 MiB of a main thread (2 MiB on some platforms) is
/// too little for deeply nested code. The memory is only reserved, and only
/// the part that is used gets committed.
const LINT_STACK_BYTES: usize = 256 * 1024 * 1024;

fn main() {
    let cli = Cli::parse();
    let colours = Colours::for_process(cli.color);

    let worker = std::thread::Builder::new()
        .name("malt".into())
        .stack_size(LINT_STACK_BYTES)
        .spawn(move || run(cli, colours));
    let result = match worker {
        Ok(handle) => handle
            .join()
            .unwrap_or_else(|_| Err(anyhow::anyhow!("the analyzer thread panicked"))),
        // Could not reserve the stack: fall back to the thread we are on.
        Err(_) => Err(anyhow::anyhow!("could not start the analyzer thread")),
    };
    if let Err(err) = result {
        eprintln!(
            "{}: {err:#}",
            paint(colours.stderr, colour::error(), "error")
        );
        process::exit(2);
    }
}

fn run(mut cli: Cli, colours: Colours) -> Result<()> {
    // Load configuration from .malt.toml (or --config path).
    let loaded = load_config(&cli, colours)?;
    let mut config = loaded.config;
    if cli.verbose > 0 {
        if let Some(path) = &loaded.matlab {
            eprintln!(
                "{} configuration: {}",
                paint(colours.stderr, colour::dim(), "malt:"),
                path.display()
            );
        }
        if let Some(path) = &loaded.skipped_matlab {
            eprintln!(
                "{} not using {}: a .malt.toml is present (set matlab_config in it to use the file)",
                paint(colours.stderr, colour::dim(), "malt:"),
                path.display()
            );
        }
    }
    if let Some(ignored) = &loaded.ignored {
        let before = cli.files.len();
        cli.files.retain(|file| !ignored.matches(file));
        if cli.verbose > 0 && cli.files.len() < before {
            eprintln!(
                "{} skipped {} file{} (ignoredFiles)",
                paint(colours.stderr, colour::dim(), "malt:"),
                before - cli.files.len(),
                if before - cli.files.len() == 1 {
                    ""
                } else {
                    "s"
                }
            );
        }
    }

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
    let project = project_index(&cli, colours)?;
    // `[lint.limits]`: `max_threads` is a budget for the whole process. The file
    // workers share it with the threads that run the checks of one large file, so
    // that nested parallelism never starts more threads than allowed.
    let limits = config.limits.resolve(malt_core::Limits::native());
    let (threads, per_file) = thread_plan(limits.max_threads, cli.files.len(), cli.fix);
    let serial = threads <= 1;
    let new_linter = || {
        let mut linter = Linter::shared(registry.clone());
        linter.set_inline_suppression(config.inline_suppression);
        linter.set_project_index(project.clone());
        linter.set_limits(limits);
        // A large file also spreads its own checks over the threads left to it.
        linter.set_file_check_threads(per_file, LINT_STACK_BYTES);
        linter
    };

    let started = Instant::now();
    let mut summary = Summary::default();
    if serial {
        // `--fix` writes files and reports each fix as it goes, so it works
        // through them one by one, in order.
        let mut linter = new_linter();
        for file_path in &cli.files {
            let checked = check_file(&mut linter, file_path, &cli)?;
            summary.report(file_path, checked, cli.verbose, cli.level, colours);
        }
    } else {
        lint_in_parallel(&cli, threads, new_linter, |file_path, checked| {
            summary.report(file_path, checked?, cli.verbose, cli.level, colours);
            Ok(())
        })?;
    }
    if cli.verbose > 0 {
        eprintln!(
            "{} {} file{} in {}",
            paint(colours.stderr, colour::dim(), "malt:"),
            cli.files.len(),
            if cli.files.len() == 1 { "" } else { "s" },
            format_duration(started.elapsed())
        );
    }

    if summary.total_diagnostics > 0 {
        println!(
            "\n{}",
            paint(
                colours.stdout,
                colour::emphasis(),
                format!(
                    "Found {} issue{} in {} file{}.",
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
                )
            )
        );
        if summary.fixable > 0 {
            println!(
                "{}",
                paint(
                    colours.stdout,
                    colour::success(),
                    format!("{} fixable with the `--fix` option.", summary.fixable)
                )
            );
        }
        if summary.unsafe_fixable > 0 && !cli.unsafe_fixes {
            println!(
                "{}",
                paint(
                    colours.stdout,
                    colour::success(),
                    format!(
                        "{} more with `--fix --unsafe-fixes` (these may change behaviour).",
                        summary.unsafe_fixable
                    )
                )
            );
        }
        process::exit(1);
    }

    Ok(())
}

/// How the `max_threads` budget is spent: `(file workers, check threads per
/// worker)`. One worker lints serially (`--fix`, one file, or a budget of 1) and
/// may give all of the budget to the file checks of a large file; otherwise
/// the workers share the budget evenly, so `workers * per_worker <= budget`.
fn thread_plan(budget: usize, files: usize, fix: bool) -> (usize, usize) {
    let budget = budget.max(1);
    let workers = if fix { 1 } else { budget.min(files).max(1) };
    (workers, budget / workers)
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
    /// The file was not analyzed: its NOFIL or RDERR diagnostic (it could not be
    /// read), or MALT_TOO_BIG (it is over `max_file_size` and was not read).
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
    // `max_file_size`: ask the file system for the size, so that a huge file is
    // never read into memory.
    let max_size = linter.limits().max_file_size;
    if let Ok(meta) = fs::metadata(file_path) {
        if meta.is_file() && meta.len() > max_size {
            return Ok(FileOutcome::Unreadable(
                malt_core::limits::too_big_diagnostic(meta.len(), max_size),
            ));
        }
    }
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
    fn report(
        &mut self,
        file_path: &Path,
        checked: Checked,
        verbose: u8,
        level: Level,
        colours: Colours,
    ) {
        let Checked { outcome, elapsed } = checked;
        let diagnostics = match outcome {
            FileOutcome::Unreadable(diag) => {
                if verbose > 0 {
                    eprintln!(
                        "{} skipped {} ({})",
                        paint(colours.stderr, colour::dim(), "malt:"),
                        file_path.display(),
                        diag.rule_id
                    );
                }
                print_diagnostic(file_path, &diag, colours.stdout);
                self.total_diagnostics += 1;
                self.files_with_issues += 1;
                return;
            }
            FileOutcome::Linted(mut diagnostics) => {
                diagnostics.retain(|d| level.shows(d.severity));
                diagnostics
            }
        };
        let prefix = paint(colours.stderr, colour::dim(), "malt:");
        match verbose {
            0 => {}
            1 => eprintln!("{prefix} linted {}", file_path.display()),
            _ => eprintln!(
                "{prefix} linted {} in {} ({} finding{})",
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
            print_diagnostic(file_path, diag, colours.stdout);
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
fn print_diagnostic(file_path: &Path, diag: &Diagnostic, on: bool) {
    let (severity, style) = match diag.severity {
        Severity::Error => ("E", colour::error()),
        Severity::Warning => ("W", colour::warning()),
        Severity::Info => ("I", colour::info()),
    };
    println!(
        "{}{} {} {}: {}",
        paint(on, colour::emphasis(), file_path.display()),
        paint(on, colour::dim(), format!(":{}:{}", diag.line, diag.column)),
        paint(on, style, format!("[{severity}]")),
        paint(on, colour::emphasis(), diag.rule_id),
        diag.message,
    );
}

/// The search path of this run as a [`ProjectIndex`], when it is on: `--index`, or
/// `[index] enabled` unless `--no-index` says otherwise. Its warnings go to stderr.
/// The `[index]` table is read (and must be valid) even when the index is off.
fn project_index(cli: &Cli, colours: Colours) -> Result<Option<Arc<dyn ProjectIndex>>> {
    let settings = path_index::load_settings(cli.config.as_deref())?;
    let on = if cli.index {
        true
    } else if cli.no_index {
        false
    } else {
        settings.enabled
    };
    if !on {
        return Ok(None);
    }
    let (index, warnings) = path_index::open(settings)?;
    for warning in &warnings {
        eprintln!(
            "{} index: {warning}",
            paint(colours.stderr, colour::warning(), "warning:")
        );
    }
    Ok(Some(Arc::new(index)))
}

/// The MATLAB Code Analyzer configuration that a project uses, relative to the
/// working directory: the file `codeIssues` reads.
const MATLAB_CONFIG_PATH: &str = "resources/codeAnalyzerConfiguration.json";

/// The configuration of this run and where it came from.
struct Loaded {
    config: Config,
    /// The `ignoredFiles` patterns of the MATLAB configuration in use.
    ignored: Option<IgnoredFiles>,
    /// The MATLAB configuration in use (for `-v`).
    matlab: Option<PathBuf>,
    /// A MATLAB configuration that exists but is not used (for `-v`).
    skipped_matlab: Option<PathBuf>,
}

/// Discover and load the configuration.
///
/// Priority:
/// 1. Explicit `--config` path: a `.json` file is a MATLAB configuration, anything
///    else a `.malt.toml` (error if not found).
/// 2. `.malt.toml` in the current working directory. Its `[lint] matlab_config`, if
///    set, names a MATLAB configuration that the TOML layers over.
/// 3. `resources/codeAnalyzerConfiguration.json` in the current working directory.
/// 4. Nothing: the defaults.
///
/// Rule and parameter problems are reported on stderr. With `--strict-config`, any
/// problem is an error instead.
fn load_config(cli: &Cli, colours: Colours) -> Result<Loaded> {
    let strict = cli.strict_config;
    if let Some(path) = cli.config.as_deref() {
        if !path.exists() {
            anyhow::bail!("config file not found: {}", path.display());
        }
        if is_json(path) {
            return matlab_only(path, strict, colours);
        }
        return toml_only(path, strict, colours);
    }

    let toml_path = Path::new(".malt.toml");
    let matlab_path = Path::new(MATLAB_CONFIG_PATH);
    if toml_path.exists() {
        let toml_config = read_toml(toml_path, strict, colours)?;
        return match toml_config.matlab_config.clone() {
            Some(name) => {
                let path = PathBuf::from(name);
                let matlab = read_matlab(&path, strict, colours)?;
                Ok(Loaded {
                    ignored: IgnoredFiles::new(&path, matlab.ignored_files),
                    config: malt_core::matlab_config::layer(matlab.config, toml_config),
                    matlab: Some(path),
                    skipped_matlab: None,
                })
            }
            None => Ok(Loaded {
                config: toml_config,
                ignored: None,
                matlab: None,
                skipped_matlab: matlab_path.is_file().then(|| matlab_path.to_path_buf()),
            }),
        };
    }
    if matlab_path.is_file() {
        return matlab_only(matlab_path, strict, colours);
    }
    Ok(Loaded {
        config: Config::default(),
        ignored: None,
        matlab: None,
        skipped_matlab: None,
    })
}

fn is_json(path: &Path) -> bool {
    path.extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
}

/// A `.malt.toml` alone.
fn toml_only(path: &Path, strict: bool, colours: Colours) -> Result<Loaded> {
    Ok(Loaded {
        config: read_toml(path, strict, colours)?,
        ignored: None,
        matlab: None,
        skipped_matlab: None,
    })
}

/// A MATLAB configuration alone.
fn matlab_only(path: &Path, strict: bool, colours: Colours) -> Result<Loaded> {
    let matlab = read_matlab(path, strict, colours)?;
    Ok(Loaded {
        ignored: IgnoredFiles::new(path, matlab.ignored_files),
        config: matlab.config,
        matlab: Some(path.to_path_buf()),
        skipped_matlab: None,
    })
}

/// Read a `.malt.toml`, reporting its rule and parameter problems.
fn read_toml(path: &Path, strict: bool, colours: Colours) -> Result<Config> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read config: {}", path.display()))?;
    let config = Config::from_toml(&content)
        .with_context(|| format!("failed to parse config: {}", path.display()))?;
    let warnings = malt_rules::config_check::config_warnings(&config);
    for warning in &warnings {
        eprintln!(
            "{} {}: {warning}",
            paint(colours.stderr, colour::warning(), "warning:"),
            path.display()
        );
    }
    if strict && !warnings.is_empty() {
        anyhow::bail!(
            "{} problem(s) in {} (--strict-config)",
            warnings.len(),
            path.display()
        );
    }
    Ok(config)
}

/// Read a MATLAB Code Analyzer configuration. A file MATLAB refuses is an error; its
/// warnings are reported on stderr, and `--strict-config` makes them an error.
fn read_matlab(
    path: &Path,
    strict: bool,
    colours: Colours,
) -> Result<malt_core::matlab_config::MatlabConfig> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read config: {}", path.display()))?;
    let matlab = match malt_core::matlab_config::parse(&content) {
        Ok(matlab) => matlab,
        Err(errors) => anyhow::bail!(
            "invalid MATLAB configuration {}:\n  {}",
            path.display(),
            errors.join("\n  ")
        ),
    };
    for warning in &matlab.warnings {
        eprintln!(
            "{} {}: {warning}",
            paint(colours.stderr, colour::warning(), "warning:"),
            path.display()
        );
    }
    if strict && !matlab.warnings.is_empty() {
        anyhow::bail!(
            "{} problem(s) in {} (--strict-config)",
            matlab.warnings.len(),
            path.display()
        );
    }
    Ok(matlab)
}

#[cfg(test)]
mod tests {
    use super::thread_plan;

    #[test]
    fn the_thread_budget_is_never_exceeded() {
        for budget in 1..=16 {
            for files in 0..=40 {
                for fix in [false, true] {
                    let (workers, per_file) = thread_plan(budget, files, fix);
                    assert!(workers >= 1 && per_file >= 1);
                    assert!(workers * per_file <= budget, "{budget} {files} {fix}");
                }
            }
        }
    }

    #[test]
    fn a_budget_of_one_is_single_threaded() {
        assert_eq!(thread_plan(1, 100, false), (1, 1));
        assert_eq!(thread_plan(1, 1, true), (1, 1));
    }

    #[test]
    fn few_files_leave_threads_to_their_checks() {
        assert_eq!(thread_plan(8, 1, false), (1, 8));
        assert_eq!(thread_plan(8, 2, false), (2, 4));
        assert_eq!(thread_plan(8, 100, false), (8, 1));
        assert_eq!(thread_plan(8, 100, true), (1, 8));
    }
}
