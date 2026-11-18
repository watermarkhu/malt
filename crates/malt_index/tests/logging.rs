//! The index's log records (through the `log` facade). This file has one test binary, so
//! the one logger it sets is ours alone.

use std::path::Path;
use std::sync::Mutex;

use log::{Level, LevelFilter, Log, Metadata, Record};
use malt_index::{parse_file, parse_source, Index, ParserConfig};

/// Records of this binary: (level, message).
static RECORDS: Mutex<Vec<(Level, String)>> = Mutex::new(Vec::new());

struct Capture;

impl Log for Capture {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        RECORDS
            .lock()
            .unwrap()
            .push((record.level(), record.args().to_string()));
    }

    fn flush(&self) {}
}

static CAPTURE: Capture = Capture;

/// The tests share the global logger and its records, so they run one at a time.
static SERIAL: Mutex<()> = Mutex::new(());

fn install() {
    let _ = log::set_logger(&CAPTURE);
    log::set_max_level(LevelFilter::Trace);
}

fn take() -> Vec<(Level, String)> {
    std::mem::take(&mut *RECORDS.lock().unwrap())
}

#[test]
fn parsing_logs_a_debug_record_and_recovered_syntax_errors_warn() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    install();
    let dir = std::env::temp_dir().join(format!("malt-log-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("Broken.m");
    std::fs::write(&file, "function y = f(x)\n  y = x + ;\nend\n").unwrap();

    take();
    parse_file(&file, &ParserConfig::default()).unwrap();
    let records = take();
    assert!(
        records
            .iter()
            .any(|(l, m)| *l == Level::Debug && m.contains("parsing")),
        "{records:?}"
    );
    assert!(
        records.iter().any(|(l, m)| *l == Level::Warn
            && m.contains("syntax errors")
            && m.contains("Broken.m")),
        "{records:?}"
    );

    take();
    parse_source(Path::new("ok.m"), b"x = 1;\n", &ParserConfig::default()).unwrap();
    assert!(take().iter().all(|(l, _)| *l != Level::Warn));
}

#[test]
fn an_unreadable_path_warns_and_is_kept_as_an_error() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    install();
    let missing = std::env::temp_dir().join("malt-log-no-such-folder-xyz");
    take();
    let index = Index::scan(std::slice::from_ref(&missing), &ParserConfig::default());
    assert_eq!(index.errors().len(), 1);
    let records = take();
    assert!(
        records
            .iter()
            .any(|(l, m)| *l == Level::Warn && m.contains("cannot read")),
        "{records:?}"
    );
}
