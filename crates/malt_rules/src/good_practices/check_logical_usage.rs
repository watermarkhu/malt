//! Logical-usage checks (BDLGI, BDLOG1, BDLOG2, BDSCI).
//!
//! These checks reason about variables used in boolean contexts (bare `if x` /
//! `while x` conditions) and short-circuit operators. They consume the
//! conservative [`TypeEnv`] built by `crate::analysis::typing` and only fire
//! on provable cases:
//!
//! - BDLGI  — the condition variable was assigned from an arithmetic operator.
//! - BDLOG1 — a transposed condition (`if a'`); MATLAB reports no other form.
//! - BDLOG2 — the condition value is scalar but not provably logical.
//! - BDSCI  — the condition variable was assigned from an array-producing
//!   expression (`a:b`, `[...]`, `{...}`).
//!
//! All diagnostics are `Severity::Warning` and gated on `disabled_checks`.

use std::collections::HashMap;

use super::*;
use crate::analysis::typing::{TypeEnv, TypeKind};

/// Per-name assignment classifications collected by scanning every assignment
/// in the file. Each map records the classification of the *latest* assignment
/// to the name, mirroring the type environment's last-assignment-wins rule.
struct AssignmentClasses {
    /// Names whose latest assignment RHS is an arithmetic operator.
    arith: HashMap<String, bool>,
    /// Names whose latest assignment RHS is provably non-scalar.
    nonscalar: HashMap<String, bool>,
    /// Names whose latest assignment RHS is a number literal.
    scalar_literal: HashMap<String, bool>,
}

impl GoodPracticesEngine {
    /// BDLGI / BDLOG1 / BDLOG2 / BDSCI: file-level logical-usage
    /// checks driven by the type environment.
    pub(crate) fn check_logical_usage(
        &self,
        tree: &tree_sitter::Tree,
        source: &str,
    ) -> Vec<Diagnostic> {
        let any_enabled = self.is_check_enabled("BDLGI")
            || self.is_check_enabled("BDLOG1")
            || self.is_check_enabled("BDLOG2")
            || self.is_check_enabled("BDSCI");
        if !any_enabled {
            return Vec::new();
        }

        let env = TypeEnv::build(tree, source);
        let mut diagnostics = Vec::new();

        // ---- Pass 1: classify every simple-identifier assignment ----------
        // Last assignment wins, mirroring the TypeEnv's semantics.
        let mut classes = AssignmentClasses {
            arith: HashMap::new(),
            nonscalar: HashMap::new(),
            scalar_literal: HashMap::new(),
        };
        let mut assignments = Vec::new();
        collect_nodes_of_kind(tree.root_node(), "assignment", &mut assignments);
        for assign in assignments {
            let Some(lhs) = assign.child_by_field_name("left") else {
                continue;
            };
            if lhs.kind() != "identifier" {
                continue;
            }
            let name = node_text(lhs, source).to_string();
            let Some(rhs) = assign.child_by_field_name("right") else {
                continue;
            };
            classes
                .arith
                .insert(name.clone(), is_arithmetic_rhs(rhs, source));
            classes
                .nonscalar
                .insert(name.clone(), is_nonscalar_rhs(rhs, source));
            classes
                .scalar_literal
                .insert(name.clone(), rhs.kind() == "number");
        }

        // ---- Pass 2: walk if/while/elseif/switch condition expressions ----
        let mut conditions = Vec::new();
        collect_condition_expressions(tree.root_node(), &mut conditions);
        for (cond, is_if_while) in conditions {
            check_bare_condition(
                cond,
                is_if_while,
                &env,
                &classes,
                source,
                self,
                &mut diagnostics,
            );
        }

        // ---- Pass 3: BDSCA removed (no MathWorks equivalent) ---------------

        diagnostics
    }
}

/// Whether an assignment RHS produces a non-logical (arithmetic) value.
///
/// Only `binary_operator` nodes whose operator is arithmetic (`+ - * / ^ \`
/// and their element-wise forms) qualify; comparisons, element-wise boolean
/// `&`/`|`, and short-circuit `&&`/`||` do not.
fn is_arithmetic_rhs(rhs: Node, source: &str) -> bool {
    if rhs.kind() != "binary_operator" {
        return false;
    }
    let op = find_operator_text(rhs, source);
    !op.is_empty() && op.chars().all(|c| "+-*/^\\.".contains(c))
}

/// Whether an assignment RHS is provably non-scalar: a colon range, or a
/// matrix/cell literal with more than one element.
fn is_nonscalar_rhs(rhs: Node, _source: &str) -> bool {
    match rhs.kind() {
        "range" => true,
        "matrix" | "cell" => !array_literal_is_scalar(rhs),
        _ => false,
    }
}

/// Whether a `matrix` / `cell` literal contains exactly one element.
fn array_literal_is_scalar(node: Node) -> bool {
    let mut count = 0usize;
    let mut stack = vec![node];
    while let Some(n) = stack.pop() {
        let mut cursor = n.walk();
        for child in n.children(&mut cursor) {
            if !child.is_named() {
                continue;
            }
            match child.kind() {
                "row" | "array" => stack.push(child),
                "number" | "string" | "identifier" | "true" | "false" | "number_constant"
                | "unary_operator" | "binary_operator" | "range" | "function_call" => {
                    count += 1;
                }
                _ => stack.push(child),
            }
        }
    }
    count == 1
}

/// Collect `(condition, is_if_while)` pairs: the condition of every
/// `if_statement`, `while_statement` and `elseif_clause`, plus the switch
/// expression of every `switch_statement` (with `is_if_while == false`).
fn collect_condition_expressions<'a>(node: Node<'a>, out: &mut Vec<(Node<'a>, bool)>) {
    match node.kind() {
        "if_statement" | "while_statement" | "elseif_clause" => {
            if let Some(cond) = first_named_child(node) {
                out.push((cond, true));
            }
        }
        "switch_statement" => {
            if let Some(expr) = first_named_child(node) {
                out.push((expr, false));
            }
        }
        _ => {}
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.is_named() {
            collect_condition_expressions(child, out);
        }
    }
}

/// Apply the bare-condition checks (BDLGI / BDLOG1 / BDLOG2 / BDSCI) to a
/// condition expression. Parentheses around a simple value are unwrapped so
/// `if (x)` behaves like `if x`.
fn check_bare_condition(
    cond: Node,
    is_if_while: bool,
    env: &TypeEnv,
    classes: &AssignmentClasses,
    source: &str,
    eng: &GoodPracticesEngine,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !is_if_while {
        return;
    }
    let mut value = cond;
    while value.kind() == "parenthesis" || value.kind() == "parenthesized_expression" {
        match first_named_child(value) {
            Some(inner) => value = inner,
            None => return,
        }
    }
    let pos = value.start_position();
    match value.kind() {
        "identifier" => {
            let name = node_text(value, source);
            let info = env.type_of(name);

            if eng.is_check_enabled("BDLGI")
                && classes.arith.get(name) == Some(&true)
                && info.kind != TypeKind::Logical
            {
                diagnostics.push(Diagnostic {
                    rule_id: "BDLGI",
                    message: "Variable might be set by a nonlogical operator.".to_string(),
                    severity: Severity::Warning,
                    byte_range: value.start_byte()..value.end_byte(),
                    line: pos.row + 1,
                    column: pos.column + 1,
                    fix: None,
                });
            }

            // MATLAB never reports BDLOG1 for a variable condition (probed with
            // non-scalar comparisons, `true(3)`, `~v` and `while`); only for a
            // transposed one, below.
            if eng.is_check_enabled("BDLOG2")
                && info.kind != TypeKind::Logical
                && (info.scalar || classes.scalar_literal.get(name) == Some(&true))
            {
                diagnostics.push(Diagnostic {
                    rule_id: "BDLOG2",
                    message: "A scalar logical value is expected in the conditional expression. \
                              Use 'any' or 'all' to reduce the array to a logical scalar, or \
                              compare the scalar value to 0."
                        .to_string(),
                    severity: Severity::Warning,
                    byte_range: value.start_byte()..value.end_byte(),
                    line: pos.row + 1,
                    column: pos.column + 1,
                    fix: None,
                });
            }

            if eng.is_check_enabled("BDSCI") && classes.nonscalar.get(name) == Some(&true) {
                diagnostics.push(Diagnostic {
                    rule_id: "BDSCI",
                    message: "Variable might be set by a nonscalar operator.".to_string(),
                    severity: Severity::Warning,
                    byte_range: value.start_byte()..value.end_byte(),
                    line: pos.row + 1,
                    column: pos.column + 1,
                    fix: None,
                });
            }
        }
        "number" if eng.is_check_enabled("BDLOG2") => {
            diagnostics.push(Diagnostic {
                rule_id: "BDLOG2",
                message: "A scalar logical value is expected in the conditional expression. \
                              Use 'any' or 'all' to reduce the array to a logical scalar, or \
                              compare the scalar value to 0."
                    .to_string(),
                severity: Severity::Warning,
                byte_range: value.start_byte()..value.end_byte(),
                line: pos.row + 1,
                column: pos.column + 1,
                fix: None,
            });
        }
        // MATLAB reports a transposed condition (`if a'`, `while a.'`) at the
        // transpose operator, whatever the type of the operand.
        "postfix_operator" if eng.is_check_enabled("BDLOG1") => {
            let op_start = value
                .named_child(0)
                .map_or(value.start_byte(), |operand| operand.end_byte());
            let line_start = source[..op_start].rfind('\n').map_or(0, |i| i + 1);
            diagnostics.push(Diagnostic {
                rule_id: "BDLOG1",
                message: "A scalar logical value is expected in the conditional expression. \
                          Use 'any' or 'all' to reduce the array to a logical scalar."
                    .to_string(),
                severity: Severity::Warning,
                byte_range: op_start..value.end_byte(),
                line: source[..op_start].matches('\n').count() + 1,
                column: op_start + 1 - line_start,
                fix: None,
            });
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn logical_ids(source: &str, check: &str) -> Vec<&'static str> {
        let tree = parse(source);
        let eng = engine();
        eng.check_logical_usage(&tree, source)
            .iter()
            .filter(|d| d.rule_id == check)
            .map(|d| d.rule_id)
            .collect()
    }

    // -- BDLGI --------------------------------------------------------------

    #[test]
    fn test_bdlgi_fires_when_arith_assigned_var_used_as_condition() {
        let source = "x = a + b;\nif x\n    y = 1;\nend\n";
        let ids = logical_ids(source, "BDLGI");
        assert_eq!(ids.len(), 1, "got: {:?}", logical_ids(source, "BDLGI"));
    }

    #[test]
    fn test_bdlgi_silent_for_logical_assignment() {
        let source = "x = a > b;\nif x\n    y = 1;\nend\n";
        assert!(logical_ids(source, "BDLGI").is_empty());
    }

    #[test]
    fn test_bdlgi_silent_for_unknown_var() {
        let source = "if x\n    y = 1;\nend\n";
        assert!(logical_ids(source, "BDLGI").is_empty());
    }

    // -- BDLOG1 -------------------------------------------------------------

    #[test]
    fn test_bdlog1_silent_for_logical_variable_conditions() {
        // MATLAB probes: `a = [1 2 3] > 1; if a`, `c = [1 2] > 0; elseif c`,
        // `c = v > 0; while c`, `c = true(3); if c`, `c = ~v; if c` and
        // `c = v > 0 && n > 0; if c` report no BDLOG1.
        for source in [
            "a = [1 2 3] > 1;\nif a\n    y = 1;\nend\n",
            "c = [1 2] > 0;\nif n == 1\n    y = 1;\nelseif c\n    y = 2;\nend\n",
            "function f(v)\nc = v > 0;\nwhile c\n    break\nend\nend\n",
            "c = true(3);\nif c\n    y = 1;\nend\n",
            "function f(v)\nc = ~v;\nif c\n    y = 1;\nend\nend\n",
            "function f(v, n)\nc = v > 0 && n > 0;\nif c\n    y = 1;\nend\nend\n",
        ] {
            assert!(logical_ids(source, "BDLOG1").is_empty(), "{source:?}");
        }
    }

    #[test]
    fn test_bdlog1_silent_for_logical_scalar() {
        let source = "x = true;\nif x\n    y = 1;\nend\n";
        assert!(logical_ids(source, "BDLOG1").is_empty());
    }

    #[test]
    fn test_bdlog1_fires_on_transposed_condition_at_the_operator() {
        // MATLAB probe on `a=1;\nif a', end; y = 2;`: BDLOG1 on line 2, column 5.
        let source = "a=1;\nif a', end; y = 2;\n";
        let diags = engine().check_logical_usage(&parse(source), source);
        let found: Vec<_> = diags.iter().filter(|d| d.rule_id == "BDLOG1").collect();
        assert_eq!(found.len(), 1, "got: {diags:?}");
        assert_eq!((found[0].line, found[0].column), (2, 5));
    }

    #[test]
    fn test_bdlog1_fires_on_transposed_condition_in_parentheses_and_while() {
        // MATLAB probes: `if (a')` gives column 6, `while a'` column 8, `if a.'` column 5.
        for (source, column) in [
            ("a=1;\nif (a')\nend\n", 6),
            ("a=1;\nwhile a'\nend\n", 8),
            ("a=[1 2];\nif a.'\nend\n", 5),
        ] {
            let diags = engine().check_logical_usage(&parse(source), source);
            let found: Vec<_> = diags.iter().filter(|d| d.rule_id == "BDLOG1").collect();
            assert_eq!(found.len(), 1, "{source:?} got: {diags:?}");
            assert_eq!((found[0].line, found[0].column), (2, column), "{source:?}");
        }
    }

    #[test]
    fn test_bdlog1_silent_for_transpose_inside_a_comparison() {
        // MATLAB probes: `if a' == 1` and `x = a';` report no BDLOG1.
        for source in ["a=1;\nif a' == 1\nend\n", "a=1;\nx = a';\n"] {
            let diags = engine().check_logical_usage(&parse(source), source);
            assert!(
                !diags.iter().any(|d| d.rule_id == "BDLOG1"),
                "{source:?} {diags:?}"
            );
        }
    }

    #[test]
    fn test_bdlog1_silent_for_numeric_condition() {
        let source = "x = 5;\nif x\n    y = 1;\nend\n";
        assert!(logical_ids(source, "BDLOG1").is_empty());
    }

    // -- BDLOG2 -------------------------------------------------------------

    #[test]
    fn test_bdlog2_fires_on_scalar_numeric_condition() {
        let source = "x = 5;\nif x\n    y = 1;\nend\n";
        let ids = logical_ids(source, "BDLOG2");
        assert_eq!(ids.len(), 1, "got: {:?}", logical_ids(source, "BDLOG2"));
    }

    #[test]
    fn test_bdlog2_fires_on_number_literal_condition() {
        let source = "if 1\n    y = 1;\nend\n";
        let ids = logical_ids(source, "BDLOG2");
        assert_eq!(ids.len(), 1, "got: {:?}", logical_ids(source, "BDLOG2"));
    }

    #[test]
    fn test_bdlog2_silent_for_logical_condition() {
        let source = "x = true;\nif x\n    y = 1;\nend\n";
        assert!(logical_ids(source, "BDLOG2").is_empty());
    }

    #[test]
    fn test_bdlog2_silent_for_non_scalar_condition() {
        let source = "x = [1 2 3];\nif x\n    y = 1;\nend\n";
        assert!(logical_ids(source, "BDLOG2").is_empty());
    }

    // -- BDSCI --------------------------------------------------------------

    #[test]
    fn test_bdsci_fires_on_range_assigned_condition() {
        let source = "x = 1:10;\nif x\n    y = 1;\nend\n";
        let ids = logical_ids(source, "BDSCI");
        assert_eq!(ids.len(), 1, "got: {:?}", logical_ids(source, "BDSCI"));
    }

    #[test]
    fn test_bdsci_fires_on_matrix_assigned_condition() {
        let source = "x = [1 2 3];\nwhile x\n    y = 1;\nend\n";
        let ids = logical_ids(source, "BDSCI");
        assert_eq!(ids.len(), 1, "got: {:?}", logical_ids(source, "BDSCI"));
    }

    #[test]
    fn test_bdsci_silent_for_scalar_assignment() {
        let source = "x = 5;\nif x\n    y = 1;\nend\n";
        assert!(logical_ids(source, "BDSCI").is_empty());
    }

    // -- config -------------------------------------------------------------

    #[test]
    fn test_logical_usage_checks_respect_disabled_checks() {
        let eng = GoodPracticesEngine {
            config: GoodPracticesConfig {
                max_variable_name_length: 63,
                disabled_checks: vec![
                    "BDLGI".to_string(),
                    "BDLOG1".to_string(),
                    "BDLOG2".to_string(),
                    "BDSCI".to_string(),
                ],
            },
        };
        let source = "x = a + b;\nif x\n    y = 1;\nend\nx = a > b;\nif x\n    y = 1;\nend\nx = 1:10;\nif x\n    y = 1;\nend\n";
        let tree = parse(source);
        assert!(eng.check_logical_usage(&tree, source).is_empty());
    }
}
