//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// SPEVC / SPLD / SPSV / SPWHOS / SPBFN / SPNF: workspace-transparency checks
    /// for `function_call` nodes inside an spmd block.
    pub(crate) fn check_spmd_function_call(
        &self,
        node: tree_sitter::Node,
        source: &str,
        nested_functions: &HashSet<String>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let Some(name) = callee_name(node, source) else {
            return;
        };
        match name.as_str() {
            "evalin" | "assignin" => {
                let scope = first_string_argument(node, source);
                if scope.as_deref() == Some("caller") {
                    self.push_diag(node, "SPEVC", "EVALIN('caller') and ASSIGNIN('caller') are invalid inside of an SPMD block.", diagnostics);
                } else if scope.as_deref() != Some("base") {
                    // 'base' is reported as SPEVB by the good-practices engine.
                    self.push_diag(node, "SPBFN", "Use of this function is invalid inside an SPMD block because it accesses or modifies the workspace in a non-transparent way.", diagnostics);
                }
            }
            "eval" | "evalc" | "clear" | "clearvars" => {
                self.push_diag(node, "SPBFN", "Use of this function is invalid inside an SPMD block because it accesses or modifies the workspace in a non-transparent way.", diagnostics);
            }
            "load" => {
                if !is_assignment_rhs(node) {
                    self.push_diag(node, "SPLD", "To avoid a transparency violation, assign the output of LOAD to a variable in SPMD blocks.", diagnostics);
                }
            }
            "save" => {
                if !has_string_argument(node, source, "-fromstruct") {
                    self.push_diag(
                        node,
                        "SPSV",
                        "SAVE cannot be called in an SPMD block without the '-fromstruct' option.",
                        diagnostics,
                    );
                }
            }
            "who" | "whos" => {
                if !has_string_argument(node, source, "-file") {
                    self.push_diag(node, "SPWHOS", "Using \"who\" or \"whos\" without \"-file\" is invalid inside an SPMD block because it accesses the workspace in a non-transparent way.", diagnostics);
                }
            }
            other => {
                if nested_functions.contains(other) {
                    self.push_diag(
                        node,
                        "SPNF",
                        &format!(
                            "The nested function {other} cannot be called from within an SPMD block."
                        ),
                        diagnostics,
                    );
                }
            }
        }
    }

    /// SPWHOS for a `who`/`whos` identifier used as an assignment value, such as
    /// `[a, b] = who;`. A bare `who` statement is a call handled by
    /// `check_spmd_function_call`.
    pub(crate) fn check_spmd_bare_who(
        &self,
        node: tree_sitter::Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !matches!(node_text(node, source), "who" | "whos") || !is_assignment_rhs(node) {
            return;
        }
        self.push_diag(node, "SPWHOS", "Using \"who\" or \"whos\" without \"-file\" is invalid inside an SPMD block because it accesses the workspace in a non-transparent way.", diagnostics);
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_spwhos_fires_bare_who_assigned_in_spmd() {
        // MATLAB probe: SPWHOS on lines 3, 4 and 5 (the bare `who` too).
        let source = "\
function foo()
    spmd
        [a, b] = who;
        c = who;
        who
    end
end
";
        let diags = check_source(source, "foo.m");
        let spwhos = filter_by_id(&diags, "SPWHOS");
        let lines: Vec<usize> = spwhos.iter().map(|d| d.line).collect();
        assert_eq!(lines, vec![3, 4, 5], "got: {diags:?}");
    }

    #[test]
    fn test_spevc_fires_in_spmd() {
        let source = "\
function foo()
    spmd
        evalin('caller', 'x');
        assignin('caller', 'x', 1);
    end
end
";
        let diags = check_source(source, "foo.m");
        let spevc = filter_by_id(&diags, "SPEVC");
        assert_eq!(
            spevc.len(),
            2,
            "SPEVC should fire for evalin/assignin('caller') in spmd, got {diags:?}"
        );
    }

    #[test]
    fn test_spevc_no_fire_outside_spmd() {
        let source = "\
function foo()
    evalin('caller', 'x');
end
";
        let diags = check_source(source, "foo.m");
        let spevc = filter_by_id(&diags, "SPEVC");
        assert!(spevc.is_empty(), "SPEVC should NOT fire outside spmd");
    }

    #[test]
    fn test_spld_fires_unassigned_load() {
        let source = "\
function foo()
    spmd
        load('data.mat');
    end
end
";
        let diags = check_source(source, "foo.m");
        let spld = filter_by_id(&diags, "SPLD");
        assert!(
            !spld.is_empty(),
            "SPLD should fire for unassigned load in spmd"
        );
    }

    #[test]
    fn test_spld_no_fire_assigned_load() {
        let source = "\
function foo()
    spmd
        d = load('data.mat');
    end
end
";
        let diags = check_source(source, "foo.m");
        let spld = filter_by_id(&diags, "SPLD");
        assert!(spld.is_empty(), "SPLD should NOT fire for assigned load");
    }

    #[test]
    fn test_spsv_fires_without_fromstruct() {
        let source = "\
function foo()
    spmd
        save('out.mat');
    end
end
";
        let diags = check_source(source, "foo.m");
        let spsv = filter_by_id(&diags, "SPSV");
        assert!(
            !spsv.is_empty(),
            "SPSV should fire for save without -fromstruct"
        );
    }

    #[test]
    fn test_spsv_no_fire_with_fromstruct() {
        let source = "\
function foo()
    spmd
        save('out.mat', '-fromstruct', s);
    end
end
";
        let diags = check_source(source, "foo.m");
        let spsv = filter_by_id(&diags, "SPSV");
        assert!(
            spsv.is_empty(),
            "SPSV should NOT fire for save with -fromstruct"
        );
    }

    #[test]
    fn test_spwhos_fires_without_file() {
        let source = "\
function foo()
    spmd
        who;
        whos;
    end
end
";
        let diags = check_source(source, "foo.m");
        let spwhos = filter_by_id(&diags, "SPWHOS");
        assert_eq!(
            spwhos.len(),
            2,
            "SPWHOS should fire for who/whos without -file"
        );
    }

    #[test]
    fn test_spwhos_no_fire_with_file() {
        let source = "\
function foo()
    spmd
        who('-file', 'x.mat');
    end
end
";
        let diags = check_source(source, "foo.m");
        let spwhos = filter_by_id(&diags, "SPWHOS");
        assert!(
            spwhos.is_empty(),
            "SPWHOS should NOT fire for who with -file"
        );
    }

    #[test]
    fn test_spbfn_fires_eval_in_spmd() {
        let source = "\
function foo()
    spmd
        eval('x');
    end
end
";
        let diags = check_source(source, "foo.m");
        let spbfn = filter_by_id(&diags, "SPBFN");
        assert_eq!(spbfn.len(), 1, "SPBFN should fire for eval in spmd");
    }

    #[test]
    fn test_spbfn_no_fire_evalin_base_in_spmd() {
        // MATLAB reports evalin('base') in spmd as SPEVB, not SPBFN.
        let source = "\
function foo()
    spmd
        evalin('base', 'x');
        assignin('base', 'x', 1);
    end
end
";
        let diags = check_source(source, "foo.m");
        assert!(filter_by_id(&diags, "SPBFN").is_empty());
    }

    #[test]
    fn test_spbfn_no_fire_outside_spmd() {
        let source = "\
function foo()
    eval('x');
end
";
        let diags = check_source(source, "foo.m");
        let spbfn = filter_by_id(&diags, "SPBFN");
        assert!(spbfn.is_empty(), "SPBFN should NOT fire outside spmd");
    }

    #[test]
    fn test_spnf_fires_nested_function_call() {
        let source = "\
function outer()
    spmd
        helper(1);
    end
    function helper(a)
    end
end
";
        let diags = check_source(source, "outer.m");
        let spnf = filter_by_id(&diags, "SPNF");
        assert!(
            !spnf.is_empty(),
            "SPNF should fire for nested function call in spmd"
        );
    }

    #[test]
    fn test_spnf_no_fire_local_function_call() {
        let source = "\
function main()
    spmd
        helper(1);
    end
end

function helper(a)
end
";
        let diags = check_source(source, "main.m");
        let spnf = filter_by_id(&diags, "SPNF");
        assert!(
            spnf.is_empty(),
            "SPNF should NOT fire for top-level local function call"
        );
    }
}
