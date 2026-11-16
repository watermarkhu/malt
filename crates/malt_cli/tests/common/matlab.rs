//! Running MATLAB from a test: one place for the binary, the skip rule and the timeout.
//!
//! The binary is `MALT_MATLAB` (a path) when set, else `matlab` on `PATH`. When neither
//! is found the test is skipped, with a message on stderr. `MALT_MATLAB_REQUIRE=1`
//! makes a missing MATLAB a failure instead (CI sets it). Calls are non-interactive
//! (`matlab -batch`), in an empty temporary working directory, and killed after
//! `MALT_MATLAB_TIMEOUT_SECS` seconds (default 600).

use std::env;
use std::ffi::OsStr;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};

/// A MATLAB installation that tests can run code in.
#[derive(Debug, Clone)]
pub struct MatlabRun {
    bin: PathBuf,
    timeout: Duration,
}

/// The MATLAB of this run, or `None` when there is none (the caller then skips).
///
/// Panics instead when `MALT_MATLAB_REQUIRE=1` and MATLAB cannot be found.
pub fn matlab() -> Option<MatlabRun> {
    let required = env::var("MALT_MATLAB_REQUIRE").is_ok_and(|v| v == "1");
    let found = match env::var_os("MALT_MATLAB") {
        Some(path) => {
            let path = PathBuf::from(path);
            path.is_file().then_some(path)
        }
        None => find_on_path("matlab"),
    };
    match found {
        Some(bin) => Some(MatlabRun {
            bin,
            timeout: timeout_from_env(),
        }),
        None if required => panic!(
            "MATLAB not found, and MALT_MATLAB_REQUIRE=1 (set MALT_MATLAB or put matlab on PATH)"
        ),
        None => {
            eprintln!(
                "SKIP: MATLAB not found (set MALT_MATLAB or put matlab on PATH; \
                 MALT_MATLAB_REQUIRE=1 turns this into a failure)"
            );
            None
        }
    }
}

impl MatlabRun {
    /// The MATLAB binary this run uses.
    pub fn bin(&self) -> &Path {
        &self.bin
    }

    /// The timeout of one call, from `MALT_MATLAB_TIMEOUT_SECS` (default 600 seconds).
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Run `code` with `matlab -batch` in a new empty temporary directory and return its
    /// standard output. A non-zero exit, or a run longer than `timeout`, is an error (the
    /// process is killed on timeout), and the error carries the end of the output.
    pub fn run_batch(&self, code: &str, timeout: Duration) -> Result<String> {
        self.run_batch_with(code, timeout, &[])
    }

    /// Like [`MatlabRun::run_batch`], with extra environment variables for MATLAB (MATLAB
    /// reads them with `getenv`).
    pub fn run_batch_with(
        &self,
        code: &str,
        timeout: Duration,
        env: &[(&str, &OsStr)],
    ) -> Result<String> {
        let dir = tempfile::tempdir().context("creating the MATLAB working directory")?;
        let mut child = Command::new(&self.bin)
            .arg("-batch")
            .arg(code)
            .envs(env.iter().copied())
            .current_dir(dir.path())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("starting {}", self.bin.display()))?;
        let mut stdout = child.stdout.take().expect("piped stdout");
        let mut stderr = child.stderr.take().expect("piped stderr");
        let out_reader = thread::spawn(move || read_all(&mut stdout));
        let err_reader = thread::spawn(move || read_all(&mut stderr));

        let deadline = Instant::now() + timeout;
        let status = loop {
            if let Some(status) = child.try_wait().context("waiting for MATLAB")? {
                break status;
            }
            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                bail!("MATLAB did not finish within {} seconds", timeout.as_secs());
            }
            thread::sleep(Duration::from_millis(100));
        };
        let stdout = out_reader.join().unwrap_or_default();
        let stderr = err_reader.join().unwrap_or_default();
        if !status.success() {
            let tail: String = format!("{stdout}{stderr}")
                .lines()
                .rev()
                .take(30)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join("\n");
            bail!("MATLAB exited with {status}:\n{tail}");
        }
        Ok(stdout)
    }
}

fn read_all(reader: &mut impl Read) -> String {
    let mut text = String::new();
    let _ = reader.read_to_string(&mut text);
    text
}

fn timeout_from_env() -> Duration {
    let secs = env::var("MALT_MATLAB_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(600);
    Duration::from_secs(secs)
}

/// The first executable called `name` (or `name.exe`) on `PATH`.
fn find_on_path(name: &str) -> Option<PathBuf> {
    let paths = env::var_os("PATH")?;
    env::split_paths(&paths)
        .flat_map(|dir| [dir.join(name), dir.join(format!("{name}.exe"))])
        .find(|candidate| is_executable(candidate))
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.is_file()
        && path
            .metadata()
            .is_ok_and(|m| m.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}
