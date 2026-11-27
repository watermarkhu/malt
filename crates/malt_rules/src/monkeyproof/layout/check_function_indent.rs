//! `MP_LAYOUT_2`: the body of a function is indented.

use malt_core::{Config, Diagnostic, FileContext};

use super::indentation::{scan, Problem};
use super::text::Text;

/// Report the first statement of a function whose body is not indented past the `function`
/// keyword. It applies to every function, at the root of a file, in a class and nested.
///
/// Only functions that start their own line are looked at, and only the first body line is
/// judged. Other indentation width problems of a body belong to `MP_LAYOUT_6`. Functions
/// with a syntax error in them are skipped.
pub(crate) fn run(_config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let text = Text::new(ctx.source, ctx.tree.root_node());
    for problem in scan(&text, ctx.tree.root_node(), 4) {
        if let Problem::FunctionBodyFlat(range) = problem {
            out.push(text.finding(
                "MP_LAYOUT_2",
                range,
                "Indent the body of a function relative to the `function` line.",
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::monkeyproof::layout::hits;

    #[test]
    fn flat_function_body_is_reported_once() {
        let d = hits(
            "MP_LAYOUT_2",
            "function y = f(x)\ny = x;\nz = 2;\nend\n",
            "",
        );
        assert_eq!(d.len(), 1, "{d:?}");
        assert_eq!((d[0].line, d[0].column), (2, 1));
    }

    #[test]
    fn indented_body_is_fine_whatever_the_width() {
        for source in [
            "function f()\n    x = 1;\nend\n",
            "function f()\n  x = 1;\nend\n",
        ] {
            assert!(hits("MP_LAYOUT_2", source, "").is_empty(), "{source:?}");
        }
    }

    #[test]
    fn functions_without_end_are_checked() {
        let d = hits(
            "MP_LAYOUT_2",
            "function f()\nx = 1;\n\nfunction g()\n    y = 1;\n",
            "",
        );
        assert_eq!(d.len(), 1, "{d:?}");
        assert_eq!(d[0].line, 2);
    }

    #[test]
    fn nested_and_method_functions_are_checked() {
        let source = "classdef A\n    methods\n        function m(obj)\n        x = 1;\n        end\n    end\nend\n";
        let d = hits("MP_LAYOUT_2", source, "");
        assert_eq!(d.len(), 1, "{d:?}");
        assert_eq!(d[0].line, 4);
    }

    #[test]
    fn one_line_and_broken_functions_are_skipped() {
        for source in [
            "function f(), x = 1; end\n",
            "function f()\nx = (1;\nend\n",
            "function f()\nend\n",
        ] {
            assert!(hits("MP_LAYOUT_2", source, "").is_empty(), "{source:?}");
        }
    }

    #[test]
    fn a_stray_dedent_is_not_this_check() {
        let d = hits("MP_LAYOUT_2", "function f()\n    x = 1;\ny = 2;\nend\n", "");
        assert!(d.is_empty(), "{d:?}");
    }
}
