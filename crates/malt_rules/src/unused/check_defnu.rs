//! DEFNU check: local function defined but never called.

use super::*;

/// Calls that reach functions by name at run time. `localfunctions` returns a
/// handle to every local function of the file (probed: a test file with
/// `functiontests(localfunctions)` gets no DEFNU for its test or helper functions).
const DYNAMIC_CALLS: &[&str] = &[
    "eval",
    "evalin",
    "evalc",
    "feval",
    "str2func",
    "localfunctions",
];

impl UnusedEngine {
    /// Run DEFNU check: local function defined but never called.
    pub(crate) fn check_defnu(
        &self,
        table: &SymbolTable,
        file_stem: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if self.is_check_disabled("DEFNU") {
            return;
        }

        // Collect all uses across all scopes to check if a local function is called.
        let all_uses: HashSet<&str> = table
            .scopes
            .iter()
            .flat_map(|s| s.uses.iter().map(|u| u.name.as_str()))
            .collect();

        // Probe: once the file calls eval, feval or str2func, MATLAB cannot tell
        // which local function is reached, and reports no DEFNU in the file.
        if DYNAMIC_CALLS.iter().any(|c| all_uses.contains(c)) {
            return;
        }

        // Probe: in a script every function is a local function, including the
        // first one, which the symbol table classifies as `Function`. Nested
        // functions are checked too (probes p5, p20, p18).
        let in_script = table
            .scopes
            .first()
            .is_some_and(|s| s.kind == ScopeKind::Script);
        // Probes (r10_D a1-a7, c1-c5, d1-d5): with a `function` line inside the
        // primary function's `arguments` block, MATLAB checks the primary function
        // like a local function: DEFNU at its name (line 1, column 14) when no call
        // reaches it, and the file name does not shadow it (probe c2).
        let open_arguments = table.arguments_unclosed;
        for (position, scope) in table.scopes.iter().enumerate() {
            let local = match scope.kind {
                ScopeKind::LocalFunction | ScopeKind::NestedFunction => true,
                ScopeKind::Function => in_script || open_arguments,
                _ => false,
            };
            if !local {
                continue;
            }
            let name = &scope.name;
            if name.is_empty() {
                continue;
            }
            if self.should_ignore_name(name) {
                continue;
            }
            // Probes: MATLAB reports no DEFNU for a script function whose name the
            // script assigned first (`scr = 1; function scr()`, with FCONF and
            // REDEF), nor for a local function named like its function file.
            // Probe ls-b__t6: a second `function t6()` is REDEF, not DEFNU.
            let duplicate = table.scopes[..position].iter().any(|s| {
                matches!(s.kind, ScopeKind::Function | ScopeKind::LocalFunction) && s.name == *name
            });
            let shadowed = duplicate
                || if in_script {
                    !table.root_scope().defs_of(name).is_empty()
                } else {
                    !open_arguments && name == file_stem
                };
            if !all_uses.contains(name.as_str()) && !shadowed {
                diagnostics.push(Diagnostic {
                    rule_id: "DEFNU",
                    message: "Function might be unused.".to_string(),
                    severity: Severity::Warning,
                    byte_range: scope.byte_range.clone(),
                    line: scope.line,
                    column: 1,
                    fix: None,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_file};
    use malt_core::Config;

    fn engine() -> Box<dyn Rule> {
        UnusedEngine::from_config(&Config::default())
    }

    // -- DEFNU: local function never called ---------------------------------

    #[test]
    fn defnu_fires_on_uncalled_local_function() {
        let diags = lint_file(
            &*engine(),
            "function main()\n    disp(1);\nend\n\nfunction helper()\n    disp(2);\nend\n",
        );
        assert!(has_id(&diags, "DEFNU"), "got: {diags:?}");
    }

    #[test]
    fn defnu_silent_on_duplicate_function_name() {
        // Probe ls-b__t6: the second `function t6()` is REDEF, not DEFNU.
        let diags = lint_file(&*engine(), "function t6()\nend\nfunction t6()\nend\n");
        assert!(!has_id(&diags, "DEFNU"), "got: {diags:?}");
    }

    #[test]
    fn defnu_silent_when_localfunctions_references_every_local_function() {
        // Probe d1 (a_tests_calc): `functiontests(localfunctions)` reaches the test
        // and the helper function; MATLAB reports neither.
        let source = "function tests = t_tests\ntests = functiontests(localfunctions);\nend\n\nfunction testA(testCase)\nend\n\nfunction helperX()\nend\n";
        let diags = lint_file(&*engine(), source);
        assert!(!has_id(&diags, "DEFNU"), "got: {diags:?}");
    }

    #[test]
    fn defnu_fires_on_test_convention_names_without_localfunctions() {
        // Probe d3: MATLAB gives no exemption to `setup` or `testX` by name in a
        // function-based test file; an unreferenced one is DEFNU.
        let source = "function tests = t_tests\ntests = 1;\nend\n\nfunction setup(testCase)\nend\n\nfunction testA(testCase)\nend\n";
        let diags = lint_file(&*engine(), source);
        assert!(has_id(&diags, "DEFNU"), "got: {diags:?}");
    }

    #[test]
    fn defnu_silent_on_a_bare_validator_name_in_arguments() {
        // Probe d5 (c_args_block:38): `{mustBeSquare}` calls the local validator.
        let source = "function y = f(M)\narguments\n    M (:,:) double {mustBeSquare}\nend\ny = numel(M);\nend\n\nfunction mustBeSquare(M)\nend\n";
        let diags = lint_file(&*engine(), source);
        assert!(!has_id(&diags, "DEFNU"), "got: {diags:?}");
    }

    #[test]
    fn defnu_fires_on_an_unused_local_validator() {
        // Probe d8: a local function that no validator or call names is DEFNU.
        let source = "function y = f(M)\narguments\n    M (1,1) double\nend\ny = M;\nend\n\nfunction isPos(M)\nend\n";
        let diags = lint_file(&*engine(), source);
        assert!(has_id(&diags, "DEFNU"), "got: {diags:?}");
    }

    #[test]
    fn defnu_ok_when_local_function_called() {
        let diags = lint_file(
            &*engine(),
            "function main()\n    helper();\nend\n\nfunction helper()\n    disp(2);\nend\n",
        );
        assert!(!has_id(&diags, "DEFNU"), "got: {diags:?}");
    }

    #[test]
    fn defnu_ok_when_function_handle_taken() {
        // Probe: a function handle counts as a use.
        let diags = lint_file(
            &*engine(),
            "function main()\n    h = @helper;\n    h();\nend\n\nfunction helper()\n    disp(2);\nend\n",
        );
        assert!(!has_id(&diags, "DEFNU"), "got: {diags:?}");
    }

    #[test]
    fn defnu_silent_when_file_calls_feval() {
        // Probe: `dead` is not reported once the file calls feval, even though
        // the feval argument is a handle or a variable.
        let diags = lint_file(
            &*engine(),
            "function r = main(name)\n    r = feval(name);\nend\n\nfunction r = dead()\n    r = 2;\nend\n",
        );
        assert!(!has_id(&diags, "DEFNU"), "got: {diags:?}");
    }

    #[test]
    fn defnu_silent_when_file_calls_eval_or_str2func() {
        // Probe: the same holds for eval and str2func.
        for call in ["eval('x = 1;');", "g = str2func('helper');"] {
            let src =
                format!("function main()\n    {call}\nend\n\nfunction dead()\n    disp(3);\nend\n");
            let diags = lint_file(&*engine(), &src);
            assert!(!has_id(&diags, "DEFNU"), "{call}: got: {diags:?}");
        }
    }

    #[test]
    fn defnu_silent_when_file_calls_evalin_or_evalc() {
        // Probes p6 and p7: MATLAB reports no DEFNU for `sub` after evalin or evalc.
        for call in ["evalin('base', 'x');", "s = evalc('disp(1)');"] {
            let src = format!("function p()\n{call}\nend\n\nfunction s = sub()\ns = 1;\nend\n");
            let diags = lint_file(&*engine(), &src);
            assert!(!has_id(&diags, "DEFNU"), "{call}: got: {diags:?}");
        }
    }

    #[test]
    fn defnu_not_silenced_by_run() {
        // Probe p8: `run` does not reach local functions, so `sub` is reported.
        let diags = lint_file(
            &*engine(),
            "function p()\nrun('foo.m');\nend\n\nfunction s = sub()\ns = 1;\nend\n",
        );
        assert!(has_id(&diags, "DEFNU"), "got: {diags:?}");
    }

    #[test]
    fn defnu_silent_when_eval_is_in_arguments_validator() {
        // Probe p10: an eval inside an arguments validator counts as a dynamic call.
        let diags = lint_file(
            &*engine(),
            "function r = p(a)\narguments\n  a {mustBeNumeric, eval('1')}\nend\nr = a;\nend\n\nfunction s = sub()\ns = 1;\nend\n",
        );
        assert!(!has_id(&diags, "DEFNU"), "got: {diags:?}");
    }

    #[test]
    fn defnu_silent_when_script_assigns_function_name() {
        // Probe scr2 (fixture ls-c__scr2): `scr2 = 1;` before `function scr2()`.
        let diags = lint_file(&*engine(), "scr2 = 1;\nfunction scr2()\nend\n");
        assert!(!has_id(&diags, "DEFNU"), "got: {diags:?}");
    }

    #[test]
    fn defnu_silent_when_function_file_has_local_function_named_as_file() {
        // Probe fconfB: a subfunction named like its function file is not reported.
        let source = "function r = fconfB(x)\nr = x;\nend\nfunction fconfB()\nend\n";
        let tree = crate::test_util::parse(source);
        let ctx = FileContext {
            tree: &tree,
            source,
            file_path: std::path::Path::new("fconfB.m"),
        };
        let diags = engine().check_file(&ctx);
        assert!(!has_id(&diags, "DEFNU"), "got: {diags:?}");
    }

    #[test]
    fn defnu_fires_on_uncalled_nested_function() {
        // Probes p5 and p20: a nested function nobody calls is reported.
        let diags = lint_file(
            &*engine(),
            "function r = p(x)\nr = x;\nfunction q = inner()\nq = 1;\nend\nend\n",
        );
        assert!(has_id(&diags, "DEFNU"), "got: {diags:?}");
    }

    #[test]
    fn defnu_ok_when_parent_calls_nested_function() {
        // Probe p15: the parent calls the nested function.
        let diags = lint_file(
            &*engine(),
            "function r = p(x)\nr = inner(x);\nfunction q = inner(y)\nq = y;\nend\nend\n",
        );
        assert!(!has_id(&diags, "DEFNU"), "got: {diags:?}");
    }

    #[test]
    fn defnu_fires_in_script_on_uncalled_function() {
        // Probes p1 and p2: in a script the first function is a local function too.
        let diags = lint_file(&*engine(), "x = 1;\nfunction h()\n  disp(1)\nend\n");
        assert!(has_id(&diags, "DEFNU"), "got: {diags:?}");
    }

    #[test]
    fn defnu_ok_in_script_when_called_from_script() {
        // Probe p12: `a` is called from the script, `b` from `a`.
        let diags = lint_file(
            &*engine(),
            "a();\nfunction a()\n  b();\nend\nfunction b()\nend\n",
        );
        assert!(!has_id(&diags, "DEFNU"), "got: {diags:?}");
    }

    #[test]
    fn defnu_not_suppressed_by_field_store_of_same_name() {
        // Probe (performance__t1): `s.f(end+1) = i` stores into a field `f`, which
        // is not a call to the local function `f`.
        let diags = lint_file(
            &*engine(),
            "function main()\n  s.f(end+1) = 1;\nend\nfunction f()\nend\n",
        );
        assert!(has_id(&diags, "DEFNU"), "got: {diags:?}");
    }

    #[test]
    fn defnu_for_function_after_classdef() {
        // Probe (naming__e): a function after a classdef `end` is a local function.
        let diags = lint_file(&*engine(), "classdef Cl\nend\nfunction helper()\nend\n");
        assert!(has_id(&diags, "DEFNU"), "got: {diags:?}");
    }

    fn defnu_lines(src: &str) -> Vec<usize> {
        lint_file(&*engine(), src)
            .iter()
            .filter(|d| d.rule_id == "DEFNU")
            .map(|d| d.line)
            .collect()
    }

    #[test]
    fn defnu_fires_on_primary_when_arguments_block_is_open() {
        // Fixture t__fv4: a `function` line inside the arguments block. MATLAB
        // reports the primary function at line 1 (uncalled) and the nested one at 5.
        let src = "function r = fvA(n, m)\n    arguments\n        n (1,1) {mustBeNumeric} = nargin\n        m = max(nargin, 2)\n        function q = inner(), q = 1; end\n    end\n    r = 1;\nend\n";
        assert_eq!(defnu_lines(src), vec![1, 5]);
    }

    #[test]
    fn defnu_primary_silent_when_called_with_open_arguments_block() {
        // Probe a2: a later function calls the primary function, so only the
        // uncalled caller (line 9) and the nested function (line 4) are reported.
        let src = "function r = fvA(n)\n    arguments\n        n (1,1) double\n        function q = inner(), q = 1; end\n    end\n    r = 1;\nend\n\nfunction z = caller()\n    z = fvA(1);\nend\n";
        assert_eq!(defnu_lines(src), vec![4, 9]);
    }

    #[test]
    fn defnu_primary_silent_without_parse_error() {
        // Probe a4: the arguments block is left open, but the file has no
        // parse error. MATLAB keeps the primary function a file function (FNDEF,
        // no DEFNU at line 1); the nested function is reported at line 4.
        let src = "function r = fvC(n)\n    arguments\n        n (1,1) double\n        function q = inner()\n    end\n    r = 1;\nend\n";
        assert_eq!(defnu_lines(src), vec![4]);
    }

    #[test]
    fn defnu_primary_not_shadowed_by_file_name_with_open_arguments_block() {
        // Probe c2: the primary function named like its file is still DEFNU.
        let src = "function r = c2(n)\n    arguments\n        n (1,1) double\n        function q = inner(), q = 1; end\n    end\n    r = 1;\nend\n";
        assert_eq!(defnu_lines(src), vec![1, 4]);
    }
}
