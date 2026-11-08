//! # 4.5-B: Input-argument renames and class/scripting syntax
//!
//! These generic compatibility checks detect removed/renamed input arguments
//! and removed class/scripting syntax that the function-name lookup table
//! cannot match:
//!
//! - **FPRENAME/XPCRENAME/SLRTRENAME/PSRENAME/DCRENAME/SERENAME** — removed
//!   string input arguments (`'fixpoint'`, `'xpc'`, `'slrt'`, `'powersys'`,
//!   `'distcomp'`, `'simevents'`).
//! - **HHCNA/HHCWE** — removed region input arguments (`'North America'`,
//!   `'Western Europe'`).
//! - **NOV6** — `'v6'` format argument.
//! - **FROPT/FROPTX** — removed print options `'-dill'`, `'-adobecset'`.
//! - **RESOU** — a `resources` folder reference (reserved folder).
//! - **REPUDD** — `schema.*` class-definition functions.
//! - **MCATP** — `@` used for class property restriction.
//! - **FGREN/FGREM** — removed `'Renderer'`/`'RendererMode'` name-value args.

use super::super::CompatibilityEngine;
use malt_core::{Diagnostic, Severity};
use tree_sitter::Node;

/// Removed string input arguments → (check ID, message).
const REMOVED_ARGS: &[(&str, &str, &str)] = &[
    ("fixpoint", "FPRENAME", "Input argument 'fixpoint' will be removed in a future release. Use 'fixedpoint' instead."),
    ("xpc", "XPCRENAME", "Input argument 'xpc' will be removed in a future release. Use 'slrealtime' instead."),
    ("slrt", "SLRTRENAME", "Input argument 'slrt' will be removed in a future release. Use 'slrealtime' instead."),
    ("powersys", "PSRENAME", "Input argument 'powersys' will be removed in a future release. Use 'sps' instead."),
    ("distcomp", "DCRENAME", "Input argument 'distcomp' will be removed in a future release. Use 'parallel' instead."),
    ("simevents", "SERENAME", "Input argument 'simevents' will be removed in a future release. Use 'slde' instead."),
    ("North America", "HHCNA", "Input argument 'North America' has been removed. Use 'hrn:here:data::olp-here-had:here-hdlm-protobuf-na-2' instead."),
    ("Western Europe", "HHCWE", "Input argument 'Western Europe' has been removed. Use 'hrn:here:data::olp-here-had:here-hdlm-protobuf-weu-2' instead."),
    ("v6", "NOV6", "'v6' will be removed in a future release. There is no simple replacement for this."),
    ("-dill", "FROPT", "'-dill' has been removed. Use Encapsulated PostScript instead."),
    ("-adobecset", "FROPTX", "'-adobecset' has been removed. There is no simple replacement for this."),
];

/// RESOU message.
const RESOU_MSG: &str = "'resources' is a reserved folder. Running MATLAB files located in a folder named 'resources' is not supported.";
/// REPUDD message.
const REPUDD_MSG: &str = "Classes defined using schema.m files are no longer supported. Use MATLAB Classes defined using the classdef keyword instead.";
/// MCATP message.
const MCATP_MSG: &str = "Using an @ sign to specify a class property restriction is unsupported and has been removed. Use property validation syntax instead.";
/// FGREN message.
const FGREN_MSG: &str = "'Renderer' will be removed in a future release and currently has no effect. There is no simple replacement for this.";
/// FGREM message.
const FGREM_MSG: &str = "'RendererMode' will be removed in a future release and currently has no effect. There is no simple replacement for this.";

/// Dispatch a node to the relevant check.
pub(crate) fn collect_checks(
    engine: &CompatibilityEngine,
    node: Node,
    source: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match node.kind() {
        "string" => engine.check_removed_arg(node, source, diagnostics),
        "command" => engine.check_command(node, source, diagnostics),
        "command_argument" => engine.check_command_arg(node, source, diagnostics),
        "field_expression" => engine.check_schema(node, source, diagnostics),
        "arguments" => engine.check_at_restriction(node, diagnostics),
        _ => {}
    }
}

impl CompatibilityEngine {
    /// Removed string input arguments / print options (function-call form).
    fn check_removed_arg(&self, node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        let text = unquoted(node, source);
        for (name, id, msg) in REMOVED_ARGS {
            if text == *name {
                push_diag(id, msg, node, Severity::Warning, diagnostics);
                return;
            }
        }
        // 'Renderer' / 'RendererMode' name-value args on figure/set/get.
        if text == "Renderer" && call_includes(node, "figure", source) {
            push_diag("FGREN", FGREN_MSG, node, Severity::Warning, diagnostics);
        } else if text == "RendererMode" {
            push_diag("FGREM", FGREM_MSG, node, Severity::Warning, diagnostics);
        }
    }

    /// Removed arguments in command form (`slrt 'slrt'`).
    fn check_command(&self, node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        // Command arguments are child `command_argument` nodes; handled there.
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "command_argument" {
                self.check_command_arg(child, source, diagnostics);
            }
        }
    }

    /// A command_argument: `'fixpoint'`, `resources`, `-dill`, etc.
    fn check_command_arg(&self, node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        let text = node_text(node, source);
        let stripped = text.trim().trim_matches('\'').trim_matches('"');
        for (name, id, msg) in REMOVED_ARGS {
            if stripped == *name {
                push_diag(id, msg, node, Severity::Warning, diagnostics);
                return;
            }
        }
        // RESOU: `resources` folder reference (cd / addpath / import).
        if stripped == "resources" || text.contains("resources/") || text.contains("resources\\") {
            push_diag("RESOU", RESOU_MSG, node, Severity::Error, diagnostics);
        }
    }

    /// REPUDD: `schema.prop(...)` / `schema(...)` class-definition calls.
    fn check_schema(&self, node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        let text = &source[node.start_byte()..node.end_byte()];
        if text.starts_with("schema.") || text.starts_with("schemaType") || text == "schema" {
            push_diag("REPUDD", REPUDD_MSG, node, Severity::Error, diagnostics);
        }
    }

    /// MCATP: an `@` sign used for property restriction in arguments.
    fn check_at_restriction(&self, node: Node, diagnostics: &mut Vec<Diagnostic>) {
        // Look for an `@` token followed by a type name (not a function handle
        // `@(args)...`). In an arguments block, `x @double` appears as an `@`
        // token between the name and a type.
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "@" {
                // A bare `@Type` restriction: next sibling is an identifier/type.
                if let Some(next) = child.next_named_sibling() {
                    if matches!(next.kind(), "identifier") {
                        push_diag("MCATP", MCATP_MSG, node, Severity::Error, diagnostics);
                        return;
                    }
                }
            }
        }
    }
}

/// Is `node` inside a call whose callee matches `name`?
fn call_includes(node: Node, name: &str, source: &str) -> bool {
    let mut cur = node.parent();
    while let Some(p) = cur {
        if p.kind() == "function_call" {
            if let Some(cn) = p.child_by_field_name("name") {
                let text = &source[cn.start_byte()..cn.end_byte()];
                if text == name {
                    return true;
                }
            }
        }
        cur = p.parent();
    }
    false
}

/// Raw text of a node.
fn node_text<'a>(node: Node<'a>, source: &'a str) -> &'a str {
    &source[node.start_byte()..node.end_byte()]
}

/// Strip surrounding quotes from a string node.
fn unquoted<'a>(node: Node<'a>, source: &'a str) -> &'a str {
    let text = &source[node.start_byte()..node.end_byte()];
    let bytes = text.as_bytes();
    if bytes.len() >= 2
        && ((bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\'')
            || (bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"'))
    {
        &text[1..text.len() - 1]
    } else {
        text
    }
}

/// Push a diagnostic at the node's location.
fn push_diag(
    id: &'static str,
    message: &str,
    node: Node,
    severity: Severity,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let pos = node.start_position();
    diagnostics.push(Diagnostic {
        rule_id: id,
        message: message.to_string(),
        severity,
        byte_range: node.start_byte()..node.end_byte(),
        line: pos.row + 1,
        column: pos.column + 1,
        fix: None,
    });
}

#[cfg(test)]
mod tests {
    use crate::compatibility::CompatibilityEngine;
    use crate::test_util::{has_id, lint_file};
    use malt_core::Config;

    fn engine() -> Box<dyn malt_core::Rule> {
        CompatibilityEngine::from_config(&Config::default())
    }

    #[test]
    fn fprename_fixpoint_fires() {
        let d = lint_file(&*engine(), "x = fixpoint('fixpoint');\n");
        assert!(has_id(&d, "FPRENAME"), "got: {d:?}");
    }

    #[test]
    fn fprename_other_arg_not_fires() {
        let d = lint_file(&*engine(), "x = fixpoint('foo');\n");
        assert!(!has_id(&d, "FPRENAME"), "got: {d:?}");
    }

    #[test]
    fn xpcrename_xpc_fires() {
        let d = lint_file(&*engine(), "xpc('xpc');\n");
        assert!(has_id(&d, "XPCRENAME"), "got: {d:?}");
    }

    #[test]
    fn slrtrename_slrt_command_fires() {
        let d = lint_file(&*engine(), "slrt 'slrt'\n");
        assert!(has_id(&d, "SLRTRENAME"), "got: {d:?}");
    }

    #[test]
    fn hhcna_north_america_fires() {
        let d = lint_file(&*engine(), "hrn('North America', 'Western Europe');\n");
        assert!(has_id(&d, "HHCNA"), "got: {d:?}");
        assert!(has_id(&d, "HHCWE"), "got: {d:?}");
    }

    #[test]
    fn nov6_v6_fires() {
        let d = lint_file(&*engine(), "save('file.mat', 'v6');\n");
        assert!(has_id(&d, "NOV6"), "got: {d:?}");
    }

    #[test]
    fn fropt_dill_fires() {
        let d = lint_file(&*engine(), "print('-dill', 'file.eps');\n");
        assert!(has_id(&d, "FROPT"), "got: {d:?}");
    }

    #[test]
    fn resou_resources_command_fires() {
        let d = lint_file(&*engine(), "cd resources\n");
        assert!(has_id(&d, "RESOU"), "got: {d:?}");
    }

    #[test]
    fn repudd_schema_prop_fires() {
        let d = lint_file(&*engine(), "schema.prop('MyClass', 'prop', 'double');\n");
        assert!(has_id(&d, "REPUDD"), "got: {d:?}");
    }

    #[test]
    fn fgren_renderer_figure_fires() {
        let d = lint_file(&*engine(), "figure('Renderer', 'painters');\n");
        assert!(has_id(&d, "FGREN"), "got: {d:?}");
    }

    #[test]
    fn fgrem_renderermode_fires() {
        let d = lint_file(&*engine(), "set(gcf, 'RendererMode', 'manual');\n");
        assert!(has_id(&d, "FGREM"), "got: {d:?}");
    }
}
