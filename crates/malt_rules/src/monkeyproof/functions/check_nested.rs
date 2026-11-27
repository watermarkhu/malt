//! FCNS-16: use nested functions sparingly.
//!
//! The standard gives no count, so each function defined inside another function is
//! reported. A local function, which sits at file level after the main function, is not
//! nested. Whether a nested function is really beneficial is for the author to judge, for
//! example to avoid a long argument list; switch the check off per file or per line then.

use malt_core::{Config, Diagnostic, FileContext};

use super::common::{function_name, functions, name_range, FnKind};
use crate::monkeyproof::finding;

const ID: &str = "MP_FCNS_16";

/// Run the check over a file.
pub(crate) fn run(_config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    for func in functions(ctx.tree.root_node()) {
        if func.kind == FnKind::Nested {
            out.push(finding(
                ctx.source,
                ID,
                name_range(func.node),
                format!(
                    "Function {} is nested in another function. Use a local function unless sharing variables is needed.",
                    function_name(func.node, ctx.source)
                ),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::common::tests_util::run;
    use super::ID;

    #[test]
    fn a_nested_function_fires_at_its_name() {
        let d = run(ID, "function f()\nfunction g()\nend\nend\n", "");
        assert_eq!(d.len(), 1, "{d:?}");
        assert_eq!((d[0].line, d[0].column), (2, 10));
    }

    #[test]
    fn local_functions_and_methods_are_fine() {
        assert!(run(ID, "function f()\nend\nfunction g()\nend\n", "").is_empty());
        let class = "classdef C\nmethods\nfunction m(obj)\nend\nend\nend\nfunction h()\nend\n";
        assert!(run(ID, class, "").is_empty());
    }

    #[test]
    fn a_function_nested_in_a_method_fires() {
        let class = "classdef C\nmethods\nfunction m(obj)\nfunction h()\nend\nend\nend\nend\n";
        assert_eq!(run(ID, class, "").len(), 1);
    }

    #[test]
    fn broken_code_does_not_panic() {
        run(ID, "function f(\nfunction g(\n", "");
    }
}
