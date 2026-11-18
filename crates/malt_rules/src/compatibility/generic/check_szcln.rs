//! # SZCLN: Colon operands must be scalar (`size` used as a range endpoint)
//!
//! A colon range whose first or last operand is a single-argument `size(X)`
//! call, as in `1:size(A)`, `size(A):3`, `1:2:size(A)` or `1:size(A(:,1))`.
//! Probed against MATLAB R2026b: every form above fires at the `size` call and
//! the message always names `'size(A)'`. Silent: `size(A,1)` (two arguments),
//! `size(A)+1` and `(size(A))` (not a bare operand), `numel(size(A))`,
//! `size(A,1):size(A,2)`, `for k = size(A)` (not a range), and the step operand
//! of `start:step:stop`, which is not checked.
//!
//! ## Check IDs
//!
//! | Check ID | Severity | Fix | Description |
//! | --- | --- | --- | --- |
//! | SZCLN | error | no | Colon operands must now be scalar, but 'size(A)' usually returns a nonscalar value. To revert to the old behavior, use 'size(A,1)' instead, or index into the first element of the variable assigned to 'size(A)'. |

use super::super::CompatibilityEngine;
use malt_core::{Diagnostic, Severity};
use tree_sitter::Node;

const SZCLN_MSG: &str = "Colon operands must now be scalar, but 'size(A)' usually returns a nonscalar value. To revert to the old behavior, use 'size(A,1)' instead, or index into the first element of the variable assigned to 'size(A)'.";

/// Collects SZCLN diagnostics for a colon range node.
pub(crate) fn collect_checks(
    _engine: &CompatibilityEngine,
    node: Node,
    source: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if node.kind() != "range" {
        return;
    }
    let mut cursor = node.walk();
    let operands: Vec<Node> = node.named_children(&mut cursor).collect();
    if operands.len() < 2 {
        return;
    }
    // The first and last operands are the range ends; a middle operand is the step.
    for operand in [operands[0], operands[operands.len() - 1]] {
        if !is_single_arg_size_call(operand, source) {
            continue;
        }
        let pos = operand.start_position();
        diagnostics.push(Diagnostic {
            rule_id: "SZCLN",
            message: SZCLN_MSG.to_string(),
            severity: Severity::Error,
            byte_range: operand.start_byte()..operand.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: None,
        });
    }
}

/// True for a `size(X)` call with exactly one argument (no dimension).
fn is_single_arg_size_call(node: Node, source: &str) -> bool {
    if node.kind() != "function_call" {
        return false;
    }
    let Some(name) = node.named_child(0) else {
        return false;
    };
    if name.kind() != "identifier" || &source[name.start_byte()..name.end_byte()] != "size" {
        return false;
    }
    let mut cursor = node.walk();
    let Some(args) = node.children(&mut cursor).find(|c| c.kind() == "arguments") else {
        return false;
    };
    (0..args.named_child_count())
        .filter_map(|i| args.named_child(i))
        .filter(|c| c.kind() != "comment")
        .count()
        == 1
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
    fn szcln_fires_on_size_range_endpoints() {
        let d = lint_file(&*engine(), "A = rand(3,4);\nx = 1:size(A);\n");
        assert!(has_id(&d, "SZCLN"), "got: {d:?}");
        let szcln = d.iter().find(|x| x.rule_id == "SZCLN").unwrap();
        assert_eq!((szcln.line, szcln.column), (2, 7));

        let d = lint_file(&*engine(), "A = rand(3,4);\nx = size(A):3;\n");
        assert!(has_id(&d, "SZCLN"), "got: {d:?}");

        let d = lint_file(&*engine(), "A = rand(3,4);\nx = 1:2:size(A);\n");
        let szcln = d.iter().find(|x| x.rule_id == "SZCLN").unwrap();
        assert_eq!((szcln.line, szcln.column), (2, 9));

        let d = lint_file(&*engine(), "A = rand(3,4);\nx = 1:size(A(:,1));\n");
        assert!(has_id(&d, "SZCLN"), "got: {d:?}");
    }

    #[test]
    fn szcln_silent_on_dimension_argument_and_non_operand_forms() {
        for src in [
            "A = rand(3,4);\nx = 1:size(A,1);\n",
            "A = rand(3,4);\nx = 1:size(A)+1;\n",
            "A = rand(3,4);\nx = 1:(size(A));\n",
            "A = rand(3,4);\nx = 1:numel(size(A));\n",
            "A = rand(3,4);\nx = size(A,1):size(A,2);\n",
            "A = rand(3,4);\nfor k = size(A)\n    disp(k);\nend\n",
            "A = rand(3,4);\nx = 1:numel(A);\n",
        ] {
            let d = lint_file(&*engine(), src);
            assert!(!has_id(&d, "SZCLN"), "{src}: got {d:?}");
        }
    }
}
