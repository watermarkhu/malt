//! Dump the index of one or more MATLAB folders as JSON, in the schema of
//! `docs/scripts/maxx_dump.py`:
//!
//! ```text
//! cargo run -p malt-index --example dump -- PATH [PATH ...] > rust.json
//! ```
//!
//! The first PATH is the root: entity paths are relative to it.

use std::path::PathBuf;
use std::process::ExitCode;

use malt_index::{dump::dump, Index, ParserConfig};

fn main() -> ExitCode {
    let paths: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    if paths.is_empty() {
        eprintln!("usage: dump PATH [PATH ...]");
        return ExitCode::from(2);
    }
    let index = Index::scan(&paths, &ParserConfig::default());
    for err in index.errors() {
        eprintln!("warning: {err}");
    }
    let root = match std::fs::canonicalize(&paths[0]) {
        Ok(root) => root,
        Err(err) => {
            eprintln!("error: {}: {err}", paths[0].display());
            return ExitCode::FAILURE;
        }
    };
    match dump(&index, &root) {
        Ok(value) => match serde_json::to_string_pretty(&value) {
            Ok(text) => {
                println!("{text}");
                ExitCode::SUCCESS
            }
            Err(err) => {
                eprintln!("error: {err}");
                ExitCode::FAILURE
            }
        },
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}
