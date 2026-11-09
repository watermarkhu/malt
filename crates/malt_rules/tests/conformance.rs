//! Source-level conformance checks for the rule engines.
//!
//! These read the rule sources and guard the bug classes that kept coming back:
//! two engines reporting the same check ID, an emitted ID that no engine
//! declares, and a message that still contains an unfilled `VAR_*` placeholder.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

fn rule_sources() -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(&path, out);
            } else if matches!(
                path.extension().and_then(|e| e.to_str()),
                Some("rs" | "toml")
            ) {
                out.push(path);
            }
        }
    }
    let mut files = Vec::new();
    walk(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut files,
    );
    files.sort();
    files
}

fn rel(path: &Path) -> String {
    path.strip_prefix(env!("CARGO_MANIFEST_DIR"))
        .unwrap_or(path)
        .display()
        .to_string()
}

fn is_check_id(s: &str) -> bool {
    !s.is_empty()
        && s.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
}

/// IDs declared in a module's `//! | ID | severity | ...` table.
fn docstring_ids(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| {
            let rest = line.strip_prefix("//! | ")?;
            let mut cells = rest.split(" | ");
            let id = cells.next()?.trim();
            let severity = cells.next()?.trim();
            (is_check_id(id) && matches!(severity, "error" | "warning" | "info"))
                .then(|| id.to_string())
        })
        .collect()
}

/// IDs declared in a data file as `id = "..."`.
fn data_ids(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| {
            let id = line.strip_prefix("id = \"")?.strip_suffix('"')?;
            is_check_id(id).then(|| id.to_string())
        })
        .collect()
}

/// IDs given to a diagnostic as a string literal in non-test code.
fn emitted_ids(text: &str) -> Vec<String> {
    let code = text.split("\n#[cfg(test)]").next().unwrap_or(text);
    let mut ids = Vec::new();
    for marker in ["rule_id: \"", "push_diag(\"", "make_diag(\"", "diag(\""] {
        for part in code.split(marker).skip(1) {
            if let Some(end) = part.find('"') {
                let id = &part[..end];
                if is_check_id(id) && id.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
                    ids.push(id.to_string());
                }
            }
        }
    }
    ids
}

#[test]
fn a_check_id_is_declared_by_one_engine_only() {
    let mut owners: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for file in rule_sources() {
        if file.extension().is_some_and(|e| e == "rs") {
            let text = fs::read_to_string(&file).unwrap();
            for id in docstring_ids(&text) {
                owners.entry(id).or_default().insert(rel(&file));
            }
        }
    }
    let duplicated: Vec<String> = owners
        .iter()
        .filter(|(_, files)| files.len() > 1)
        .map(|(id, files)| format!("{id}: {files:?}"))
        .collect();
    assert!(
        duplicated.is_empty(),
        "IDs declared in more than one engine (the finding would be reported twice):\n{}",
        duplicated.join("\n")
    );
}

#[test]
fn every_emitted_id_is_declared() {
    let mut declared: BTreeSet<String> = BTreeSet::new();
    let mut emitted: BTreeMap<String, String> = BTreeMap::new();
    for file in rule_sources() {
        let text = fs::read_to_string(&file).unwrap();
        if file.extension().is_some_and(|e| e == "toml") {
            declared.extend(data_ids(&text));
        } else {
            declared.extend(docstring_ids(&text));
            for id in emitted_ids(&text) {
                emitted.entry(id).or_insert_with(|| rel(&file));
            }
        }
    }
    // `NOSEMI` is a standalone rule: its module has a front-matter block, not a
    // check table.
    declared.insert("NOSEMI".to_string());

    let undeclared: Vec<String> = emitted
        .iter()
        .filter(|(id, _)| !declared.contains(*id))
        .map(|(id, file)| format!("{id} ({file})"))
        .collect();
    assert!(
        undeclared.is_empty(),
        "IDs emitted but missing from every engine's check table: {undeclared:?}"
    );
}

/// Files whose `VAR_*` text is a template that the code fills in at use.
const TEMPLATE_FILES: &[&str] = &[
    "src/performance/mod.rs",
    "src/system_objects/mod.rs",
    "src/compatibility/generic/check_forward_gates.rs",
    "src/data/compatibility.toml",
    "src/data/suggested_improvements.toml",
];

#[test]
fn messages_have_no_unfilled_placeholders() {
    let placeholders = [
        "VAR_NAME",
        "VAR_NUMBER",
        "VAR_FILE",
        "VAR_OPERATOR",
        "VAR_RESERVED_WORD",
    ];
    let mut found = Vec::new();
    for file in rule_sources() {
        let name = rel(&file).replace('\\', "/");
        if TEMPLATE_FILES.contains(&name.as_str()) {
            continue;
        }
        let text = fs::read_to_string(&file).unwrap();
        let code = text.split("\n#[cfg(test)]").next().unwrap_or(&text);
        for (i, line) in code.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") {
                continue;
            }
            if placeholders.iter().any(|p| line.contains(p)) {
                found.push(format!("{name}:{}: {}", i + 1, trimmed));
            }
        }
    }
    assert!(
        found.is_empty(),
        "unfilled placeholders (fill them, or list the file in TEMPLATE_FILES if it is a template):\n{}",
        found.join("\n")
    );
}
