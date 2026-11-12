//! # FINSNI / FINSNO: `nargin` and `nargout` used as values in a script
//!
//! MATLAB no longer supports `nargin` or `nargout` in a script file (one whose
//! first statement is not `function`). Only the bare name is reported: the call
//! `nargin('f')` is a different function and is not reported. Probed for
//! `nargout`: a bare statement, an assignment, an `if` condition and an argument
//! are each reported at the name's own column, and the same names in a function
//! file are silent.

use malt_core::{Diagnostic, Severity};
use tree_sitter::Node;

/// The script-only names, their check IDs and MATLAB's messages.
const SCRIPT_NAMES: &[(&str, &str, &str)] = &[
    (
        "nargin",
        "FINSNI",
        "Support for 'nargin' in a script has been removed.",
    ),
    (
        "nargout",
        "FINSNO",
        "Support for 'nargout' in a script has been removed.",
    ),
];

/// Report a bare `nargin` or `nargout` identifier in a script file.
pub(crate) fn collect_checks(node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
    // A bare statement `nargout` parses as a command; the name is then the whole command.
    let is_command = node.kind() == "command";
    if node.kind() != "identifier" && !is_command {
        return;
    }
    if node.kind() == "identifier" && node.parent().is_some_and(|p| p.kind() == "command") {
        return;
    }
    let name = source[node.start_byte()..node.end_byte()].trim();
    let Some(&(_, id, message)) = SCRIPT_NAMES.iter().find(|&&(n, _, _)| n == name) else {
        return;
    };
    // The name of a call (`nargin('f')`) is not a bare `nargin`.
    if let Some(parent) = node.parent() {
        let is_call_name = parent.kind() == "function_call"
            && parent
                .child_by_field_name("name")
                .is_some_and(|name| name.id() == node.id());
        if is_call_name {
            return;
        }
        // Assigning to the name (`nargout = 2`) does not read the function's count.
        let is_target = parent.kind() == "assignment"
            && parent
                .child_by_field_name("left")
                .is_some_and(|left| left.id() == node.id());
        if is_target {
            return;
        }
    }
    if !in_script(node) {
        return;
    }
    let start = node.start_position();
    diagnostics.push(Diagnostic {
        rule_id: id,
        message: message.to_string(),
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

    fn fires(source: &str, id: &str) -> bool {
        let engine: Box<dyn Rule> = CompatibilityEngine::from_config(&Config::default());
        has_id(&lint_file(engine.as_ref(), source), id)
    }

    #[test]
    fn bare_nargin_fires_in_a_script() {
        assert!(fires("x = nargin;\n", "FINSNI"));
        assert!(fires("n = nargchk(1, 2, nargin);\n", "FINSNI"));
    }

    #[test]
    fn nargin_call_and_function_files_do_not_fire() {
        assert!(!fires("y = nargin('f');\n", "FINSNI"));
        assert!(!fires("function r = f(a)\nr = nargin;\nend\n", "FINSNI"));
    }

    #[test]
    fn bare_nargout_fires_in_a_script() {
        // Probed in MATLAB: FINSNO as a statement, in an assignment, an `if`
        // condition and an argument.
        for src in [
            "nargout\n",
            "n = nargout;\n",
            "if nargout > 0\n    disp(1)\nend\n",
            "disp(nargout)\n",
        ] {
            assert!(fires(src, "FINSNO"), "{src}");
        }
    }

    #[test]
    fn nargout_call_and_function_file_do_not_fire() {
        // Silent for the call forms (`nargout(...)`) and in a function file.
        for src in [
            "nOut = max(0, nargout(task.Fcn));\n",
            "x = nargout('max');\n",
            "function r = f()\nr = nargout;\nend\n",
        ] {
            assert!(!fires(src, "FINSNO"), "{src}");
        }
    }
}
