//! FXSETL: a `for` loop reuses the loop variable of an enclosing `for` loop,
//! so the inner loop overwrites the outer loop variable. Probed: nested loops,
//! nesting through an `if` or `while`, and three levels all fire at the inner
//! iterator name. Silent: sibling loops, loops over different names, and a loop
//! body that only captures the variable in an anonymous function.

use super::*;

/// The identifier bound by a `for` loop's iterator.
fn loop_variable(node: Node) -> Option<Node> {
    let iterator = find_child_of_kind(node, "iterator")?;
    first_named_child(iterator).filter(|n| n.kind() == "identifier")
}

impl GoodPracticesEngine {
    /// FXSETL: the iterator of a `for` loop has the same name as the iterator of
    /// an enclosing `for` loop.
    pub(crate) fn check_fxsetl(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        // A parfor nested in a for with the same variable is reported (probed: fx04);
        // an outer parfor is not (fx05 gives PFXST instead).
        if !self.is_check_enabled("FXSETL") || node.kind() != "for_statement" {
            return Vec::new();
        }
        let Some(var) = loop_variable(node) else {
            return Vec::new();
        };
        let name = node_text(var, source);
        let shadows_outer = std::iter::successors(node.parent(), |n| n.parent()).any(|a| {
            a.kind() == "for_statement"
                && !is_parfor_node(a, source)
                && loop_variable(a).is_some_and(|v| node_text(v, source) == name)
        });
        if !shadows_outer {
            return Vec::new();
        }

        let pos = var.start_position();
        vec![Diagnostic {
            rule_id: "FXSETL",
            message: format!(
                "'{name}' is also the loop variable for an outer loop. Rename this variable, so this loop does not affect the outer loop variable."
            ),
            severity: Severity::Warning,
            byte_range: var.start_byte()..var.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: None,
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::has_id;

    /// Run FXSETL on every node of `src`.
    fn fxsetl_diags(src: &str) -> Vec<Diagnostic> {
        let tree = parse(src);
        let eng = engine();
        let mut out = Vec::new();
        let mut stack = vec![tree.root_node()];
        while let Some(n) = stack.pop() {
            out.extend(eng.check_fxsetl(n, src));
            let mut c = n.walk();
            stack.extend(n.children(&mut c));
        }
        out
    }

    #[test]
    fn fxsetl_fires_on_nested_loop_with_same_variable() {
        let diags = fxsetl_diags("for i = 1:3\n    for i = 1:2\n        x = i;\n    end\nend\n");
        assert_eq!(diags.len(), 1, "got: {diags:?}");
        assert_eq!(diags[0].rule_id, "FXSETL");
        assert_eq!((diags[0].line, diags[0].column), (2, 9));
        assert!(diags[0]
            .message
            .starts_with("'i' is also the loop variable"));
    }

    #[test]
    fn fxsetl_fires_through_if_while_and_deeper_nesting() {
        let src = "for i = 1:2\n  if true\n    for i = 1:2\n    end\n  end\nend\n";
        assert!(has_id(&fxsetl_diags(src), "FXSETL"));
        let src = "for i = 1:2\n  while true\n    for i = 1:3\n      break;\n    end\n    break;\n  end\nend\n";
        assert!(has_id(&fxsetl_diags(src), "FXSETL"));
        let src = "for i = 1:2\n  for j = 1:2\n    for i = 1:2\n    end\n  end\nend\n";
        assert_eq!(fxsetl_diags(src).len(), 1);
    }

    #[test]
    fn fxsetl_fires_on_parfor_nested_in_for() {
        // Probed: `for i ... parfor i ...` gives FXSETL at the parfor's `i` (fx04, column 12).
        let diags = fxsetl_diags(
            "function f()\nfor i = 1:3\n    parfor i = 1:2\n        x = i;\n    end\nend\nend\n",
        );
        assert_eq!(diags.len(), 1, "got: {diags:?}");
        assert_eq!((diags[0].line, diags[0].column), (3, 12));
    }

    #[test]
    fn fxsetl_silent_on_sibling_and_different_loops() {
        for src in [
            "for i = 1:2\n  x = i;\nend\nfor i = 1:3\n  x = i;\nend\n",
            "for i = 1:3\n  for j = 1:2\n    x = i + j;\n  end\nend\n",
            "for i = 1:2\n  f = @() i;\n  for j = 1:2\n  end\nend\n",
        ] {
            assert!(!has_id(&fxsetl_diags(src), "FXSETL"), "{src}");
        }
    }
}
