//! Dump the index of one or more MATLAB folders as JSON, in the schema of
//! `docs/scripts/maxx_dump.py`:
//!
//! ```text
//! cargo run -p malt-index --example dump -- [--live] PATH [PATH ...] > rust.json
//! cargo run -p malt-index --example dump -- --plaintext DIR > rust.json
//! ```
//!
//! `--live` reads the `.mlx` files on the path as live scripts (as maxx's
//! `parse_live_scripts`). `--plaintext DIR` reads each `.m` file of DIR as plain-text live
//! code, with the members keyed by file stem (as maxx's `LiveScriptParser.parse_plaintext`).
//! The first PATH is the root: entity paths are relative to it.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use malt_index::dump::{dump, entity_json};
use malt_index::{parse_live_text_file, Index, ParserConfig};
use serde_json::{json, Map};

fn main() -> ExitCode {
    let mut args: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let live = args.first().is_some_and(|a| a == "--live");
    if live {
        args.remove(0);
    }
    let plaintext = args.first().is_some_and(|a| a == "--plaintext");
    if plaintext {
        args.remove(0);
    }
    if args.is_empty() {
        eprintln!("usage: dump [--live | --plaintext] PATH [PATH ...]");
        return ExitCode::from(2);
    }
    let root = match std::fs::canonicalize(&args[0]) {
        Ok(root) => root,
        Err(err) => {
            eprintln!("error: {}: {err}", args[0].display());
            return ExitCode::FAILURE;
        }
    };
    let value = if plaintext {
        plaintext_members(&root)
    } else {
        let config = ParserConfig {
            parse_live_scripts: live,
            ..ParserConfig::default()
        };
        let index = Index::scan(&args, &config);
        for err in index.errors() {
            eprintln!("warning: {err}");
        }
        dump(&index, &root).map_err(|e| e.to_string())
    };
    match value.and_then(|v| serde_json::to_string_pretty(&v).map_err(|e| e.to_string())) {
        Ok(text) => {
            println!("{text}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

/// The `.m` files of `dir`, each read as plain-text live code, keyed by stem.
fn plaintext_members(dir: &Path) -> Result<serde_json::Value, String> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "m") && p.is_file())
        .collect();
    files.sort();
    let mut members = Map::new();
    for file in files {
        let entity = parse_live_text_file(&file).map_err(|e| e.to_string())?;
        let stem = entity.name.clone();
        members.insert(stem, entity_json(&entity, dir));
    }
    Ok(json!({
        "root": dir.to_string_lossy(),
        "members": members,
    }))
}
