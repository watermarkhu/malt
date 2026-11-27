//! NAMING-14: variables and functions named like MATLAB built-in functions.

use std::collections::HashSet;

use malt_core::{Config, Diagnostic, FileContext};
use serde::Deserialize;

use super::decls::{declarations, Kind};
use crate::monkeyproof::finding;

const ID: &str = "MP_NAMING_14";

/// Parameters of `MP_NAMING_14`.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
struct Params {
    /// Names to treat as built-in on top of the built-in list (for a toolbox or
    /// the project's own functions).
    additional_names: Vec<String>,
    /// Names to accept although they are on the list.
    allowed_names: Vec<String>,
}

/// Core MATLAB functions that are often shadowed by accident. Not the whole of MATLAB:
/// malt has no list of the installed functions.
const BUILTINS: &[&str] = &[
    "abs",
    "accumarray",
    "acos",
    "all",
    "angle",
    "any",
    "area",
    "asin",
    "atan",
    "atan2",
    "axes",
    "axis",
    "bar",
    "bitand",
    "bitor",
    "bitshift",
    "bitxor",
    "box",
    "cat",
    "ceil",
    "cell",
    "cellfun",
    "char",
    "circshift",
    "class",
    "clc",
    "clear",
    "clock",
    "close",
    "colormap",
    "conj",
    "contour",
    "conv",
    "cos",
    "cosh",
    "cross",
    "cummax",
    "cummin",
    "cumprod",
    "cumsum",
    "date",
    "datetime",
    "deal",
    "deconv",
    "det",
    "diag",
    "diff",
    "dir",
    "disp",
    "display",
    "dot",
    "double",
    "eig",
    "eps",
    "error",
    "exist",
    "exp",
    "eye",
    "factor",
    "factorial",
    "false",
    "fft",
    "fieldnames",
    "figure",
    "filter",
    "find",
    "fix",
    "fliplr",
    "flipud",
    "floor",
    "fopen",
    "format",
    "fprintf",
    "func2str",
    "gamma",
    "get",
    "getfield",
    "grid",
    "height",
    "hist",
    "histc",
    "hold",
    "ifft",
    "image",
    "imag",
    "inf",
    "Inf",
    "input",
    "int16",
    "int32",
    "int64",
    "int8",
    "interp1",
    "intersect",
    "inv",
    "isa",
    "iscell",
    "ischar",
    "isempty",
    "isequal",
    "isfield",
    "islogical",
    "ismember",
    "isnan",
    "isnumeric",
    "isreal",
    "isstruct",
    "kron",
    "legend",
    "length",
    "line",
    "linspace",
    "load",
    "log",
    "log10",
    "log2",
    "logical",
    "lower",
    "lu",
    "mat2str",
    "max",
    "mean",
    "median",
    "mesh",
    "meshgrid",
    "min",
    "mod",
    "mode",
    "nan",
    "NaN",
    "nnz",
    "norm",
    "not",
    "now",
    "num2str",
    "numel",
    "ones",
    "or",
    "path",
    "pause",
    "pi",
    "plot",
    "plot3",
    "polyfit",
    "polyval",
    "power",
    "print",
    "prod",
    "rand",
    "randi",
    "randn",
    "randperm",
    "rank",
    "real",
    "rem",
    "repmat",
    "reshape",
    "round",
    "rows",
    "save",
    "set",
    "setdiff",
    "sign",
    "sin",
    "sinh",
    "size",
    "sort",
    "sprintf",
    "sqrt",
    "squeeze",
    "std",
    "strcat",
    "strcmp",
    "strfind",
    "string",
    "strrep",
    "strsplit",
    "strtrim",
    "struct",
    "sum",
    "surf",
    "svd",
    "table",
    "tan",
    "tanh",
    "text",
    "tic",
    "time",
    "title",
    "toc",
    "trace",
    "transpose",
    "tril",
    "triu",
    "true",
    "type",
    "union",
    "unique",
    "upper",
    "var",
    "vertcat",
    "view",
    "what",
    "which",
    "width",
    "xlabel",
    "xor",
    "ylabel",
    "zeros",
    "zlabel",
];

/// The names of the arity table (common built-in functions), read from its `name = "..."`
/// lines.
fn arity_names() -> impl Iterator<Item = &'static str> {
    include_str!("../../data/arity.toml")
        .lines()
        .filter_map(|line| line.strip_prefix("name = \""))
        .filter_map(|rest| rest.strip_suffix('"'))
}

/// Report variables and functions that take a built-in's name. Methods are skipped,
/// since overriding `disp` or `plot` in a class is normal.
pub(crate) fn run(config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let params: Params = config.rule_params(ID);
    let mut names: HashSet<&str> = BUILTINS.iter().copied().chain(arity_names()).collect();
    names.extend(params.additional_names.iter().map(String::as_str));
    for allowed in &params.allowed_names {
        names.remove(allowed.as_str());
    }
    for decl in declarations(ctx.tree.root_node(), ctx.source) {
        let what = match decl.kind {
            Kind::Variable => "variable",
            Kind::Function if !decl.is_method => "function",
            _ => continue,
        };
        if names.contains(decl.name) {
            out.push(finding(
                ctx.source,
                ID,
                decl.range,
                format!(
                    "The {what} '{}' shadows a MATLAB function of the same name; rename it.",
                    decl.name
                ),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::monkeyproof::lint;
    use crate::monkeyproof::naming::from_config;

    fn run(src: &str, toml: &str) -> Vec<(usize, usize, String)> {
        lint(from_config, src, toml)
            .into_iter()
            .filter(|d| d.rule_id == super::ID)
            .map(|d| (d.line, d.column, d.message))
            .collect()
    }

    #[test]
    fn variable_named_like_a_builtin() {
        let found = run("x = 1;\nmax = 3;\nsum = max + x;\n", "");
        assert_eq!(found.len(), 2);
        assert_eq!((found[0].0, found[0].1), (2, 1));
        assert_eq!((found[1].0, found[1].1), (3, 1));
    }

    #[test]
    fn inputs_outputs_and_loop_variables_count() {
        let src = "function size = f(length)\n    for mean = 1:3\n    end\nend\n";
        assert_eq!(run(src, "").len(), 3);
    }

    #[test]
    fn local_function_named_like_a_builtin() {
        let found = run(
            "function r = main\n    r = 1;\nend\nfunction y = norm(x)\n    y = x;\nend\n",
            "",
        );
        assert_eq!(found.len(), 1);
        assert_eq!((found[0].0, found[0].1), (4, 14));
    }

    #[test]
    fn methods_may_override() {
        let src =
            "classdef C\n    methods\n        function disp(obj)\n        end\n    end\nend\n";
        assert!(run(src, "").is_empty());
    }

    #[test]
    fn case_matters_and_plain_names_are_silent() {
        assert!(run("Max = 1;\nvalue = 2;\ni = 3;\nj = 4;\n", "").is_empty());
    }

    #[test]
    fn a_field_or_a_call_is_not_a_declaration() {
        assert!(run("s.max = 1;\nm = max(s.max);\n", "").is_empty());
    }

    #[test]
    fn parameters_are_respected() {
        let toml = "[lint.rules.MP_NAMING_14]\nadditional_names = [\"myToolboxFcn\"]\nallowed_names = [\"max\"]\n";
        assert!(run("max = 1;\n", toml).is_empty());
        assert_eq!(run("myToolboxFcn = 1;\n", toml).len(), 1);
    }

    #[test]
    fn broken_code_does_not_panic() {
        run("max = \nfunction sum(\n", "");
    }
}
