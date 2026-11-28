//! Argument-count checks (GTARG, LTARG).
//!
//! - GTARG fires when a `function_call` passes more arguments than the callee
//!   accepts ("Function might be called with too many arguments.").
//! - LTARG fires when a `function_call` passes fewer arguments than the callee
//!   requires ("Function might be called with too few arguments.").
//!
//! Resolution is two-tier:
//!
//! 1. Same-file functions: calls are matched against the functions and
//!    local functions defined in the same file (via [`FileMeta`]). A function
//!    whose last input is `varargin` accepts any number of extra arguments.
//! 2. Builtin table: calls whose name appears in `data/arity.toml` are
//!    checked against that function's declared `[min_args, max_args]` range.
//!
//! ## Documented limitation
//!
//! Cross-file resolution is not implemented (deferred): a call to a
//! function that is neither defined in the same file nor present in the
//! builtin table is skipped entirely, even if the callee is a known MATLAB
//! toolbox function with a fixed arity. Function-call nodes that are actually
//! array indexing (`A(1, 2)`) cannot be distinguished from calls by
//! tree-sitter-matlab and are subject to the same lookup rules.

use std::collections::{HashMap, HashSet};
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

/// The arity table of builtins is not used: `var(X, 0, 1, 'omitnan')` and
/// `ones(n, m, 3, "uint8")` are valid and the table says otherwise (plan 6.2). Only
/// functions of the same file are compared; the table is kept for a verified rebuild.
const USE_BUILTIN_TABLE: bool = false;

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
        // A function with an `arguments` block can have optional inputs (`x = 1`) and
        // name-value pairs, so its call counts say nothing (plan 6.2).
        let mut with_block = HashSet::new();
        let mut definitions = Vec::new();
        collect_nodes_of_kind(tree.root_node(), "function_definition", &mut definitions);
        for def in definitions {
            let mut cursor = def.walk();
            if def
                .children(&mut cursor)
                .any(|c| c.kind() == "arguments_statement")
            {
                if let Some(name) = def.child_by_field_name("name") {
                    with_block.insert(node_text(name, source).to_string());
                }
            }
        }
        for func in meta.functions.iter().chain(meta.local_functions.iter()) {
            if with_block.contains(&func.name) {
                continue;
            }
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
            } else if USE_BUILTIN_TABLE {
                match ARITY_TABLE.get(name) {
                    Some(&arity) => arity,
                    None => continue,
                }
            } else {
                continue;
            };

            // A comma-separated list argument adds an unknown number of values: the count
            // is then a lower bound, so only too many arguments can be told (plan 6.2:
            // `emitLogRecord(obj, 5, body, varargin{:})` is not too few).
            let (arg_count, cs_list) = call_argument_count(call, source);

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
            } else if arg_count < min && !cs_list && self.is_check_enabled("LTARG") {
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

/// Count the arguments of a `function_call` node, and whether one of them can
/// stand for several values (`c{:}`, `varargin{:}`, `s.f`). Comments and line
/// continuations are named nodes but not arguments; a call with empty
/// parentheses has zero arguments.
fn call_argument_count(call: Node, source: &str) -> (usize, bool) {
    let Some(args) = find_child_of_kind(call, "arguments") else {
        return (0, false);
    };
    let mut cursor = args.walk();
    let mut count = 0;
    let mut cs_list = false;
    for arg in args
        .children(&mut cursor)
        .filter(|c| c.is_named() && !matches!(c.kind(), "comment" | "line_continuation"))
    {
        count += 1;
        cs_list |= arg.kind() == "field_expression"
            || (arg.kind() != "cell" && node_text(arg, source).trim_end().ends_with('}'));
    }
    (count, cs_list)
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
    fn test_arity_skips_builtins_and_functions_with_an_arguments_block() {
        // Plan 6.2 (dogfooding): both were false positives on real code.
        assert!(arity_ids("v = var(X, 0, 1, 'omitnan');\n", "GTARG").is_empty());
        assert!(arity_ids("c = ones(2, 3, 3, 'uint8');\n", "GTARG").is_empty());
        let optional = "function caller()\n    f(1);\nend\nfunction f(a, b)\n    arguments\n        a\n        b = 1\n    end\n    disp(a + b);\nend\n";
        assert!(arity_ids(optional, "LTARG").is_empty());
    }

    #[test]
    fn test_arity_ignores_continuations_and_expands_cs_lists() {
        // Plan 6.2 (dogfooding): a `...` after a comma is not an argument, and
        // `varargin{:}` can supply any number of values.
        let callee = "function y = myfunc(a, b, c)\n    y = a + b + c;\nend\n";
        let wrapped =
            format!("function caller()\n    myfunc(1, 2, ...\n        3);\nend\n{callee}");
        assert!(arity_ids(&wrapped, "GTARG").is_empty());
        let spread =
            format!("function caller(varargin)\n    myfunc(1, varargin{{:}});\nend\n{callee}");
        assert!(arity_ids(&spread, "LTARG").is_empty());
        let many = format!("function caller(c)\n    myfunc(1, 2, 3, 4, c{{:}});\nend\n{callee}");
        assert_eq!(arity_ids(&many, "GTARG").len(), 1);
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
    fn test_builtin_calls_are_not_compared() {
        // The builtin table is switched off (`USE_BUILTIN_TABLE`): unverified, and wrong
        // for functions with name-value arguments (plan 6.2).
        assert!(arity_ids("sin(1, 2);\n", "GTARG").is_empty());
        assert!(arity_ids("disp();\n", "LTARG").is_empty());
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
