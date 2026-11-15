//! The index's log records, sent to Python's `logging` logger `malt_index`.
//!
//! `malt-index` logs through the `log` facade (no Python in the core). This module is the
//! bridge: a `log::Log` that calls `logging.getLogger("malt_index").log(level, message)`.
//! Python decides what is shown: the handler, the level and the format are the caller's.
//! A record's level maps to the Python level as `logging` numbers it (ERROR 40, WARNING 30,
//! INFO 20, DEBUG 10, and TRACE 5, below DEBUG). The bridge is installed when the module is
//! imported; if another `log` logger is already set, the records are not forwarded.

use std::sync::OnceLock;

use log::{LevelFilter, Log, Metadata, Record};
use pyo3::prelude::*;

/// The Python logger `malt_index`, set when the module is imported.
static PY_LOGGER: OnceLock<Py<PyAny>> = OnceLock::new();

/// The bridge itself (a unit struct: the logger object is in [`PY_LOGGER`]).
struct PyBridge;

static BRIDGE: PyBridge = PyBridge;

impl Log for PyBridge {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let Some(logger) = PY_LOGGER.get() else {
            return;
        };
        let level: i32 = match record.level() {
            log::Level::Error => 40,
            log::Level::Warn => 30,
            log::Level::Info => 20,
            log::Level::Debug => 10,
            log::Level::Trace => 5,
        };
        let message = record.args().to_string();
        Python::attach(|py| {
            // A failing logger must not fail the index: the record is dropped.
            let _ = logger.bind(py).call_method1("log", (level, message));
        });
    }

    fn flush(&self) {}
}

/// Install the bridge (once). Called from the module initialiser, with the GIL held.
pub(crate) fn install(py: Python<'_>) -> PyResult<()> {
    let logger = py
        .import("logging")?
        .call_method1("getLogger", ("malt_index",))?;
    let _ = PY_LOGGER.set(logger.unbind());
    if log::set_logger(&BRIDGE).is_ok() {
        log::set_max_level(LevelFilter::Trace);
    }
    Ok(())
}
