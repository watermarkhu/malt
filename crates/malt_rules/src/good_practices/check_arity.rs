//! Argument-count checks (GTARG, LTARG).
//!
//! - GTARG fires when a `function_call` passes more arguments than the callee
//!   accepts ("Function might be called with too many arguments.").
//! - LTARG fires when a `function_call` passes fewer arguments than the callee
//!   requires ("Function might be called with too few arguments.").
//!
//! Resolution is two-tier:
//!
//! 1. **Same-file functions**: calls are matched against the functions and
//!    local functions defined in the same file (via [`FileMeta`]). A function
//!    whose last input is `varargin` accepts any number of extra arguments.
//! 2. **Builtin table**: calls whose name appears in `data/arity.toml` are
//!    checked against that function's declared `[min_args, max_args]` range.
//!
//! ## Documented limitation
//!
//! Cross-file resolution is **not** implemented (deferred): a call to a
//! function that is neither defined in the same file nor present in the
//! builtin table is skipped entirely, even if the callee is a known MATLAB
//! toolbox function with a fixed arity. Function-call nodes that are actually
//! array indexing (`A(1, 2)`) cannot be distinguished from calls by
//! tree-sitter-matlab and are subject to the same lookup rules.

use std::collections::HashMap;
use std::sync::LazyLock;

use super::*;
use serde::Deserialize;

/// One entry of the builtin arity table.
#[derive(Debug, Deserialize)]
struct ArityEntry {
    /// Builtin function name.
    name: String,
    /// Minimum number of arguments the function accepts.
    min_args: usize,
    /// Maximum number of arguments the function accepts.
    max_args: usize,
}

/// Top-level structure of `data/arity.toml`.
#[derive(Debug, Deserialize)]
struct ArityData {
    /// The function entries.
    #[serde(default)]
    functions: Vec<ArityEntry>,
}

/// Raw TOML source embedded at compile time.
const ARITY_TOML_SOURCE: &str = include_str!("../data/arity.toml");

/// Parsed builtin arity table: function name → `(min_args, max_args)`.
static ARITY_TABLE: LazyLock<HashMap<String, (usize, usize)>> = LazyLock::new(|| {
    let data: ArityData =
        toml::from_str(ARITY_TOML_SOURCE).expect("failed to parse data/arity.toml");
    data.functions
        .into_iter()
        .map(|f| (f.name, (f.min_args, f.max_args)))
        .collect()
});

impl GoodPracticesEngine {
    /// GTARG / LTARG: compare every `function_call`'s argument count against
    /// the callee's declared arity (same-file functions first, then the
    /// builtin table).
    pub(crate) fn check_arity(&self, tree: &tree_sitter::Tree, source: &str) -> Vec<Diagnostic> {
        let any_enabled = self.is_check_enabled("GTARG") || self.is_check_enabled("LTARG");
        if !any_enabled {
            return Vec::new();
        }

        // Tier 1: same-file function arities (min, max).
        let meta = FileMeta::build(tree, source);
        let mut same_file: HashMap<&str, (usize, usize)> = HashMap::new();
        for func in meta.functions.iter().chain(meta.local_functions.iter()) {
            let has_varargin = func.inputs.iter().any(|i| i == "varargin");
            let min = if has_varargin {
                func.inputs.len().saturating_sub(1)
            } else {
                func.inputs.len()
            };
            let max = if has_varargin {
                usize::MAX
            } else {
                func.inputs.len()
            };
            same_file.insert(func.name.as_str(), (min, max));
        }

        let mut diagnostics = Vec::new();
        let mut calls = Vec::new();
        collect_nodes_of_kind(tree.root_node(), "function_call", &mut calls);
        for call in calls {
            let Some(name) = get_function_call_name(call, source) else {
                continue;
            };

            let (min, max) = if let Some(&arity) = same_file.get(name) {
                arity
            } else if let Some(&arity) = ARITY_TABLE.get(name) {
                arity
            } else {
                continue;
            };

            let arg_count = call_argument_count(call);

            if arg_count > max && self.is_check_enabled("GTARG") {
                let pos = call.start_position();
                diagnostics.push(Diagnostic {
                    rule_id: "GTARG",
                    message: "Function might be called with too many arguments.".to_string(),
                    severity: Severity::Warning,
                    byte_range: call.start_byte()..call.end_byte(),
                    line: pos.row + 1,
                    column: pos.column + 1,
                    fix: None,
                });
            } else if arg_count < min && self.is_check_enabled("LTARG") {
                let pos = call.start_position();
                diagnostics.push(Diagnostic {
                    rule_id: "LTARG",
                    message: "Function might be called with too few arguments.".to_string(),
                    severity: Severity::Warning,
                    byte_range: call.start_byte()..call.end_byte(),
                    line: pos.row + 1,
                    column: pos.column + 1,
                    fix: None,
                });
            }
        }

        diagnostics
    }
}

/// Count the named arguments of a `function_call` node. A call with empty
/// parentheses has zero arguments.
fn call_argument_count(call: Node) -> usize {
    find_child_of_kind(call, "arguments")
        .map(count_named_children)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arity_ids(source: &str, check: &str) -> Vec<&'static str> {
        let tree = parse(source);
        let eng = engine();
        eng.check_arity(&tree, source)
            .iter()
            .filter(|d| d.rule_id == check)
            .map(|d| d.rule_id)
            .collect()
    }

    // -- same-file functions ------------------------------------------------

    #[test]
    fn test_gtarg_fires_on_same_file_function_too_many_args() {
        let source = "\
function caller()
    myfunc(1, 2, 3);
end
function y = myfunc(a, b)
    y = a + b;
end
";
        let ids = arity_ids(source, "GTARG");
        assert_eq!(ids.len(), 1, "got: {:?}", arity_ids(source, "GTARG"));
    }

    #[test]
    fn test_ltarg_fires_on_same_file_function_too_few_args() {
        let source = "\
function caller()
    myfunc(1);
end
function y = myfunc(a, b)
    y = a + b;
end
";
        let ids = arity_ids(source, "LTARG");
        assert_eq!(ids.len(), 1, "got: {:?}", arity_ids(source, "LTARG"));
    }

    #[test]
    fn test_same_file_correct_arg_count_is_silent() {
        let source = "\
function caller()
    myfunc(1, 2);
end
function y = myfunc(a, b)
    y = a + b;
end
";
        assert!(arity_ids(source, "GTARG").is_empty());
        assert!(arity_ids(source, "LTARG").is_empty());
    }

    #[test]
    fn test_varargin_accepts_extra_args() {
        let source = "\
function caller()
    myfunc(1, 2, 3, 4);
end
function y = myfunc(a, varargin)
    y = a;
end
";
        assert!(arity_ids(source, "GTARG").is_empty());

        let source = "\
function caller()
    myfunc();
end
function y = myfunc(a, varargin)
    y = a;
end
";
        let ids = arity_ids(source, "LTARG");
        assert_eq!(ids.len(), 1, "got: {:?}", arity_ids(source, "LTARG"));
    }

    // -- builtin table ------------------------------------------------------

    #[test]
    fn test_gtarg_fires_on_builtin_too_many_args() {
        let source = "sin(1, 2);\n";
        let ids = arity_ids(source, "GTARG");
        assert_eq!(ids.len(), 1, "got: {:?}", arity_ids(source, "GTARG"));
    }

    #[test]
    fn test_ltarg_fires_on_builtin_too_few_args() {
        let source = "disp();\n";
        let ids = arity_ids(source, "LTARG");
        assert_eq!(ids.len(), 1, "got: {:?}", arity_ids(source, "LTARG"));
    }

    #[test]
    fn test_builtin_correct_calls_are_silent() {
        let source = "sin(1);\nzeros(3, 3);\nsize(m, 2);\nsum(x, 1);\nrand();\n";
        assert!(arity_ids(source, "GTARG").is_empty());
        assert!(arity_ids(source, "LTARG").is_empty());
    }

    #[test]
    fn test_unknown_callee_is_skipped() {
        let source = "someExternalFunction(1, 2, 3, 4, 5);\n";
        assert!(arity_ids(source, "GTARG").is_empty());
        assert!(arity_ids(source, "LTARG").is_empty());
    }

    // -- config -------------------------------------------------------------

    #[test]
    fn test_arity_checks_respect_disabled_checks() {
        let eng = GoodPracticesEngine {
            config: GoodPracticesConfig {
                max_variable_name_length: 63,
                disabled_checks: vec!["GTARG".to_string(), "LTARG".to_string()],
            },
        };
        let source = "sin(1, 2);\ndisp();\n";
        let tree = parse(source);
        assert!(eng.check_arity(&tree, source).is_empty());
    }
}
