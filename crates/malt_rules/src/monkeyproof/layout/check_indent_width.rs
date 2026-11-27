//! `MP_LAYOUT_6`: each nesting level is indented by four spaces.

use malt_core::{Config, Diagnostic, FileContext};
use serde::Deserialize;

use super::indentation::{scan, Problem};
use super::text::Text;

/// Parameters of `MP_LAYOUT_6`.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct Params {
    /// Spaces per nesting level.
    indent: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self { indent: 4 }
    }
}

/// Report the first line of a body whose indentation is not exactly one level deeper than
/// the construct that owns it: functions, `if`/`elseif`/`else`, loops, `switch` cases,
/// `try`/`catch`, `arguments` blocks and class blocks with their members.
///
/// One finding per body, on its first offending line. Only lines that start with spaces and
/// whose owner starts its own line are judged, so one-line forms and tab-indented lines are
/// skipped (the latter belong to `MP_LAYOUT_13`). A `case` is measured from the `case` line,
/// not from the `switch`. A function body that is not indented at all is `MP_LAYOUT_2`.
pub(crate) fn run(config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let params: Params = config.rule_params("MP_LAYOUT_6");
    let text = Text::new(ctx.source, ctx.tree.root_node());
    for problem in scan(&text, ctx.tree.root_node(), params.indent) {
        if let Problem::Width { range, expected } = problem {
            out.push(text.finding(
                "MP_LAYOUT_6",
                range,
                format!(
                    "Indent by {} spaces per level; this line should start at column {}.",
                    params.indent,
                    expected + 1
                ),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::monkeyproof::layout::hits;

    #[test]
    fn two_space_nesting_is_reported_per_body() {
        let source = "if x\n  if y\n    z = 1;\n  end\nend\n";
        let d = hits("MP_LAYOUT_6", source, "");
        assert_eq!(d.len(), 2, "{d:?}");
        assert_eq!((d[0].line, d[0].column), (2, 3));
        assert_eq!(d[1].line, 3);
    }

    #[test]
    fn four_space_nesting_is_fine() {
        let source = "function f(x)\n    if x\n        for i = 1:3\n            disp(i);\n        end\n    else\n        y = 1;\n    end\nend\n";
        assert!(hits("MP_LAYOUT_6", source, "").is_empty());
    }

    #[test]
    fn over_indented_body_is_reported() {
        let d = hits("MP_LAYOUT_6", "while x\n      x = x - 1;\nend\n", "");
        assert_eq!(d.len(), 1, "{d:?}");
        assert!(d[0].message.contains("column 5"), "{}", d[0].message);
    }

    #[test]
    fn switch_try_and_catch_bodies_are_checked() {
        let source = "switch a\n    case 1\n      x = 1;\n    otherwise\n        x = 2;\nend\ntry\n  y = 1;\ncatch\n    y = 2;\nend\n";
        let d = hits("MP_LAYOUT_6", source, "");
        let lines: Vec<_> = d.iter().map(|d| d.line).collect();
        assert_eq!(lines, [3, 8], "{d:?}");
    }

    #[test]
    fn class_blocks_and_members_are_checked() {
        let source = "classdef A\n  properties\n    P = 1\n  end\n  methods\n    function m(obj)\n        x = 1;\n    end\n  end\nend\n";
        let d = hits("MP_LAYOUT_6", source, "");
        assert!(d.iter().any(|d| d.line == 2), "{d:?}");
    }

    #[test]
    fn flat_function_body_is_the_other_check() {
        let d = hits("MP_LAYOUT_6", "function f()\nx = 1;\nend\n", "");
        assert!(d.is_empty(), "{d:?}");
    }

    #[test]
    fn one_liners_tabs_and_broken_code_are_skipped() {
        for source in [
            "if x, y = 1; end\n",
            "if x\n\ty = 1;\nend\n",
            "if x\ny = (1;\nend\n",
        ] {
            assert!(hits("MP_LAYOUT_6", source, "").is_empty(), "{source:?}");
        }
    }

    #[test]
    fn the_indent_parameter_is_respected() {
        let source = "if x\n  y = 1;\nend\n";
        let toml = "[lint.rules.MP_LAYOUT_6]\nindent = 2\n";
        assert!(hits("MP_LAYOUT_6", source, toml).is_empty());
        assert_eq!(hits("MP_LAYOUT_6", source, "").len(), 1);
    }
}
