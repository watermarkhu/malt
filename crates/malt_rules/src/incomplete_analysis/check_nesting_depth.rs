//! NOSPC: File too complex (nesting of statements).

use super::*;

/// MATLAB reports NOSPC when the weighted nesting of statements on one path
/// reaches this value (weights in `statement_weight`, in quarter levels).
const NOSPC_WEIGHT: usize = 994;

impl IncompleteAnalysisEngine {
    /// NOSPC: File too complex (statements nested too deeply).
    pub(crate) fn check_nesting_depth(
        &self,
        metrics: &TreeMetrics,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        // Only statement nesting counts: MATLAB reports no NOSPC for deep parentheses
        // or function nesting within the limits probed.
        if metrics.max_statement_weight >= NOSPC_WEIGHT {
            diagnostics.push(Diagnostic {
                rule_id: "NOSPC",
                message: "The file is too complex to analyze. Refactor the code to improve code maintainability. For example, reduce the nesting level of conditions or functions.".to_string(),
                severity: Severity::Error,
                byte_range: 0..source.len().min(1),
                line: 1,
                column: 1,
                fix: None,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::engine;
    use crate::test_util::{has_id, lint_file};

    /// Statements opened in the given order, one per kind, then the body and the `end`s.
    fn nested(kinds: &[&str]) -> String {
        let mut opened = String::new();
        for (i, kind) in kinds.iter().enumerate() {
            opened.push_str(
                match *kind {
                    "if" => "if x\n",
                    "for" => "for k{i} = 1:2\n",
                    "switch" => "switch x\ncase 1\n",
                    other => panic!("unknown kind {other}"),
                }
                .replace("{i}", &i.to_string())
                .as_str(),
            );
        }
        let ends = "end\n".repeat(kinds.len());
        format!("{opened}y = 1;\n{ends}")
    }

    // -- NOSPC: nesting depth -----------------------------------------------

    #[test]
    fn nospc_fires_on_nested_ifs_at_weight_996() {
        // MATLAB probe: 249 nested `if`s (weight 996) give NOSPC, 248 (992) do not.
        let source = nested(&vec!["if"; 249]);
        assert!(has_id(&lint_file(&*engine(), &source), "NOSPC"));
        let source = nested(&vec!["if"; 248]);
        assert!(!has_id(&lint_file(&*engine(), &source), "NOSPC"));
    }

    #[test]
    fn nospc_weights_switch_as_eight_quarters() {
        // MATLAB probe: 125 nested `switch`es (1000) give NOSPC, 124 (992) do not.
        let source = nested(&vec!["switch"; 125]);
        assert!(has_id(&lint_file(&*engine(), &source), "NOSPC"));
        let source = nested(&vec!["switch"; 124]);
        assert!(!has_id(&lint_file(&*engine(), &source), "NOSPC"));
    }

    #[test]
    fn nospc_weights_for_as_three_quarters() {
        // MATLAB probe: 246 `if`s and 3 `for`s (993) do not give NOSPC; 247 `if`s and
        // 2 `for`s (994) do.
        let mut kinds = vec!["if"; 246];
        kinds.extend(["for"; 3]);
        assert!(!has_id(&lint_file(&*engine(), &nested(&kinds)), "NOSPC"));
        let mut kinds = vec!["if"; 247];
        kinds.extend(["for"; 2]);
        assert!(has_id(&lint_file(&*engine(), &nested(&kinds)), "NOSPC"));
    }

    /// `ifs` nested `if`s around one statement with `lambdas` nested anonymous functions.
    fn nested_with_lambdas(ifs: usize, lambdas: usize) -> String {
        format!(
            "{}g = {}1;\n{}",
            "if x\n".repeat(ifs),
            "@() ".repeat(lambdas),
            "end\n".repeat(ifs)
        )
    }

    #[test]
    fn nospc_weights_lambdas_as_three_quarters() {
        // MATLAB probe: 331 nested `@()` (993) are fine, 332 (996) give NOSPC.
        let diags = lint_file(&*engine(), &nested_with_lambdas(0, 331));
        assert!(!has_id(&diags, "NOSPC"), "got: {diags:?}");
        let diags = lint_file(&*engine(), &nested_with_lambdas(0, 332));
        assert!(has_id(&diags, "NOSPC"), "got: {diags:?}");
    }

    #[test]
    fn nospc_adds_lambdas_to_the_statement_weight() {
        // MATLAB probe: 247 nested `if`s (988) with one lambda (991) are fine, with two
        // (994) NOSPC; 248 `if`s (992) with one lambda (995) give NOSPC.
        let diags = lint_file(&*engine(), &nested_with_lambdas(247, 1));
        assert!(!has_id(&diags, "NOSPC"), "got: {diags:?}");
        let diags = lint_file(&*engine(), &nested_with_lambdas(247, 2));
        assert!(has_id(&diags, "NOSPC"), "got: {diags:?}");
        let diags = lint_file(&*engine(), &nested_with_lambdas(248, 1));
        assert!(has_id(&diags, "NOSPC"), "got: {diags:?}");
    }

    #[test]
    fn nospc_weighs_the_file_function_as_five() {
        // MATLAB probe: 247 `if`s in a function (988 + 5 = 993) do not give NOSPC, 248
        // (992 + 5) do; 246 `if`s and 2 `for`s in a function (990 + 5) do, 246 `if`s and
        // 1 `for` (987 + 5) do not.
        let in_function = |body: &str| format!("function f(a)\n{body}end\n");
        let ifs = |n: usize| nested(&vec!["if"; n]);
        assert!(!has_id(
            &lint_file(&*engine(), &in_function(&ifs(247))),
            "NOSPC"
        ));
        assert!(has_id(
            &lint_file(&*engine(), &in_function(&ifs(248))),
            "NOSPC"
        ));
        let mut kinds = vec!["if"; 246];
        kinds.push("for");
        assert!(!has_id(
            &lint_file(&*engine(), &in_function(&nested(&kinds))),
            "NOSPC"
        ));
        kinds.push("for");
        assert!(has_id(
            &lint_file(&*engine(), &in_function(&nested(&kinds))),
            "NOSPC"
        ));
    }

    #[test]
    fn nospc_adds_no_weight_for_nested_functions() {
        // MATLAB probe: a function nested in the file's function with 247 `if`s is fine:
        // the outer function weighs 5, the nested one nothing (988 + 5).
        let body = nested(&vec!["if"; 247]);
        let source = format!("function f(a)\nfunction g(b)\n{body}end\nend\n");
        assert!(!has_id(&lint_file(&*engine(), &source), "NOSPC"));
    }

    #[test]
    fn nospc_ok_on_shallow_nesting() {
        let diags = lint_file(&*engine(), "if x\n    y = 1;\nend\n");
        assert!(!has_id(&diags, "NOSPC"), "got: {diags:?}");
    }

    #[test]
    fn nospc_stops_the_analysis_of_a_deep_loop_nest() {
        // MATLAB probe (400 nested `for`s with an undefined variable and a NOSEMI
        // inside): NOSPC is the only message.
        let diags = lint_file(&*engine(), &nested(&vec!["for"; 400]));
        assert_eq!(diags.len(), 1, "got: {diags:?}");
        assert!(has_id(&diags, "NOSPC"));
    }

    #[test]
    fn nospc_is_reported_for_loop_nests_past_malt_s_depth_limit() {
        // Weight 3 per `for`: 332 nested ones (996) give NOSPC, 331 (993) do not.
        let diags = lint_file(&*engine(), &nested(&vec!["for"; 332]));
        assert_eq!(diags.len(), 1, "got: {diags:?}");
        assert!(has_id(&diags, "NOSPC"));
        let diags = lint_file(&*engine(), &nested(&vec!["for"; 331]));
        assert!(!has_id(&diags, "NOSPC"), "got: {diags:?}");
    }

    #[test]
    fn deep_nests_of_each_kind_get_the_limit_not_a_crash() {
        for kinds in [vec!["if"; 2000], vec!["for"; 2000], vec!["switch"; 2000]] {
            let diags = lint_file(&*engine(), &nested(&kinds));
            assert_eq!(diags.len(), 1, "got: {diags:?}");
            assert!(has_id(&diags, "NOSPC"));
        }
    }

    #[test]
    fn mxaset_stops_the_analysis_of_a_long_operator_chain() {
        // MATLAB probe: a chain of 1,000 terms gives MXASET alone.
        let source = format!("x = {};\nq = 2\n", vec!["1"; 1000].join("+"));
        let diags = lint_file(&*engine(), &source);
        assert_eq!(diags.len(), 1, "got: {diags:?}");
        assert!(has_id(&diags, "MXASET"));
    }
}
