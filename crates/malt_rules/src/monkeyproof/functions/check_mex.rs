//! FCNS-7: mex-functions need versions for all platforms.
//!
//! A call counts as a mex call when a file `<name>.mex*` sits in the same folder as the
//! file being checked. The check lists that folder and reports the first call of each
//! such function when a platform of `platforms` has no file for it. An entry of
//! `platforms` is a mex extension, or several separated by `|` when any of them will do.
//! The default asks for 32 and 64 bit Windows, Linux, and Intel or Apple silicon macOS.
//!
//! A mex file in another folder, or reached through a path, is not looked for. Nothing is
//! reported when the folder cannot be listed.

use std::collections::BTreeSet;
use std::path::Path;

use malt_core::{Config, Diagnostic, FileContext};
use serde::Deserialize;
use tree_sitter::Node;

use crate::monkeyproof::finding;

const ID: &str = "MP_FCNS_7";

/// Parameters of `MP_FCNS_7`.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct Params {
    platforms: Vec<String>,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            platforms: ["mexw32", "mexw64", "mexa64", "mexmaci64|mexmaca64"]
                .map(String::from)
                .to_vec(),
        }
    }
}

/// Run the check over a file.
pub(crate) fn run(config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let params: Params = config.rule_params(ID);
    let folder = match ctx.file_path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => Path::new("."),
    };
    let Ok(entries) = std::fs::read_dir(folder) else {
        return;
    };
    // Extensions of the mex files in the folder, by function name.
    let mut found: std::collections::BTreeMap<String, BTreeSet<String>> = Default::default();
    for entry in entries.flatten() {
        let path = entry.path();
        let (Some(stem), Some(ext)) = (
            path.file_stem().and_then(|s| s.to_str()),
            path.extension().and_then(|s| s.to_str()),
        ) else {
            continue;
        };
        if ext.starts_with("mex") {
            found
                .entry(stem.to_string())
                .or_default()
                .insert(ext.to_string());
        }
    }
    if found.is_empty() {
        return;
    }
    let mut reported = BTreeSet::new();
    visit(
        ctx.tree.root_node(),
        ctx.source,
        &params,
        &found,
        &mut reported,
        out,
    );
}

fn visit(
    node: Node,
    source: &str,
    params: &Params,
    found: &std::collections::BTreeMap<String, BTreeSet<String>>,
    reported: &mut BTreeSet<String>,
    out: &mut Vec<Diagnostic>,
) {
    let name = match node.kind() {
        "function_call" if !is_member_call(node) => node
            .child_by_field_name("name")
            .filter(|n| n.kind() == "identifier"),
        "command" => node.child(0).filter(|n| n.kind() == "command_name"),
        _ => None,
    };
    if let Some(name_node) = name {
        let name = source.get(name_node.byte_range()).unwrap_or("");
        if let Some(extensions) = found.get(name) {
            let missing: Vec<&str> = params
                .platforms
                .iter()
                .filter(|p| !p.split('|').any(|alt| extensions.contains(alt.trim())))
                .map(String::as_str)
                .collect();
            if !missing.is_empty() && reported.insert(name.to_string()) {
                out.push(finding(
                    source,
                    ID,
                    name_node.byte_range(),
                    format!(
                        "The mex function {name} has no version for {}. Add a build for every platform.",
                        missing.join(", ")
                    ),
                ));
            }
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        visit(child, source, params, found, reported, out);
    }
}

/// `a.b(x)` or `pkg.f(x)`: the call is a member of something, not a plain function call.
fn is_member_call(call: Node) -> bool {
    call.parent().is_some_and(|p| {
        p.kind() == "field_expression"
            && p.named_child(0)
                .is_some_and(|object| object.id() != call.id())
    })
}

#[cfg(test)]
mod tests {
    use malt_core::{Config, FileContext};

    use super::*;

    fn run_in(files: &[&str], source: &str, toml: &str) -> Vec<Diagnostic> {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("malt_mp_fcns7_{}_{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        for f in files {
            std::fs::write(dir.join(f), "").unwrap();
        }
        let path = dir.join("main.m");
        std::fs::write(&path, source).unwrap();
        let tree = crate::test_util::parse(source);
        let ctx = FileContext {
            tree: &tree,
            source,
            file_path: &path,
            project: None,
        };
        let config = Config::from_toml(toml).unwrap();
        let mut out = Vec::new();
        run(&config, &ctx, &mut out);
        let _ = std::fs::remove_dir_all(&dir);
        out
    }

    #[test]
    fn a_mex_with_missing_platforms_fires_at_the_first_call() {
        let d = run_in(
            &["fast.mexw64"],
            "function f()\nx = fast(1);\ny = fast(2);\nend\n",
            "",
        );
        assert_eq!(d.len(), 1, "{d:?}");
        assert_eq!((d[0].line, d[0].column), (2, 5));
        assert!(d[0].message.contains("mexa64"));
        assert!(!d[0].message.contains("mexw64"));
    }

    #[test]
    fn a_mex_with_all_platforms_is_fine() {
        let files = [
            "fast.mexw32",
            "fast.mexw64",
            "fast.mexa64",
            "fast.mexmaca64",
        ];
        assert!(run_in(&files, "function f()\nx = fast(1);\nend\n", "").is_empty());
    }

    #[test]
    fn the_platform_list_is_a_parameter() {
        let toml = "[lint.rules.MP_FCNS_7]\nplatforms = [\"mexw64\"]\n";
        let src = "function f()\nx = fast(1);\nend\n";
        assert!(run_in(&["fast.mexw64"], src, toml).is_empty());
        assert_eq!(run_in(&["fast.mexa64"], src, toml).len(), 1);
    }

    #[test]
    fn calls_to_other_functions_and_members_are_fine() {
        let src = "function f(s)\nx = other(1);\ny = s.fast(1);\nend\n";
        assert!(run_in(&["fast.mexw64"], src, "").is_empty());
    }

    #[test]
    fn broken_code_does_not_panic() {
        run_in(&["fast.mexw64"], "function f(\nfast(\n", "");
    }
}
