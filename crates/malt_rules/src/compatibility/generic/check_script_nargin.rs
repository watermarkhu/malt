//! # FINSNI: `nargin` used as a value in a script
//!
//! MATLAB no longer supports `nargin` in a script file (one whose first
//! statement is not `function`). Only the bare name is reported: the call
//! `nargin('f')` is a different function and is not reported.

use malt_core::{Diagnostic, Severity};
use tree_sitter::Node;

/// FINSNI message (MATLAB's text).
const FINSNI_MSG: &str = "Support for 'nargin' in a script has been removed.";

/// Report a bare `nargin` identifier in a script file.
pub(crate) fn collect_checks(node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
    if node.kind() != "identifier" || &source[node.start_byte()..node.end_byte()] != "nargin" {
        return;
    }
    // The name of a call (`nargin('f')`) is not a bare `nargin`.
    if let Some(parent) = node.parent() {
        let is_call_name = parent.kind() == "function_call"
            && parent
                .child_by_field_name("name")
                .is_some_and(|name| name.id() == node.id());
        if is_call_name {
            return;
        }
    }
    if !in_script(node) {
        return;
    }
    let start = node.start_position();
    diagnostics.push(Diagnostic {
        rule_id: "FINSNI",
        message: FINSNI_MSG.to_string(),
        severity: Severity::Error,
        byte_range: node.start_byte()..node.end_byte(),
        line: start.row + 1,
        column: start.column + 1,
        fix: None,
    });
}

/// Whether the file that `node` belongs to is a script: its first statement is
/// neither a function nor a class definition.
fn in_script(node: Node) -> bool {
    let mut root = node;
    while let Some(parent) = root.parent() {
        root = parent;
    }
    let mut cursor = root.walk();
    let first = root
        .named_children(&mut cursor)
        .find(|child| child.kind() != "comment");
    first.is_some_and(|child| !matches!(child.kind(), "function_definition" | "class_definition"))
}

#[cfg(test)]
mod tests {
    use crate::compatibility::CompatibilityEngine;
    use crate::test_util::{has_id, lint_file};
    use malt_core::{Config, Rule};

    fn fires(source: &str) -> bool {
        let engine: Box<dyn Rule> = CompatibilityEngine::from_config(&Config::default());
        has_id(&lint_file(engine.as_ref(), source), "FINSNI")
    }

    #[test]
    fn bare_nargin_fires_in_a_script() {
        assert!(fires("x = nargin;\n"));
        assert!(fires("n = nargchk(1, 2, nargin);\n"));
    }

    #[test]
    fn nargin_call_and_function_files_do_not_fire() {
        assert!(!fires("y = nargin('f');\n"));
        assert!(!fires("function r = f(a)\nr = nargin;\nend\n"));
    }
}
