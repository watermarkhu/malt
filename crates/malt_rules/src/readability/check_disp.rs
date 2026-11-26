//! # DSPSP: `disp(sprintf(...))` → `fprintf`
//!
//! `check_disp` rewrites `disp(sprintf(...))` into a direct `fprintf(...)`
//! call, skipping the intermediate string construction.

use super::*;

impl ReadabilityEngine {
    /// DSPSP: `disp(sprintf(...))` → `fprintf`
    pub(crate) fn check_disp(
        &self,
        node: tree_sitter::Node,
        source: &str,
        results: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("DSPSP") {
            return;
        }

        let args = match find_arguments(node) {
            Some(a) => a,
            None => return,
        };

        let named_children: Vec<_> = (0..args.named_child_count())
            .filter_map(|i| args.named_child(i as u32))
            .collect();

        if named_children.len() != 1 {
            return;
        }

        let inner = named_children[0];
        if inner.kind() == "identifier" {
            // `disp(s)` after `s = sprintf(...)`: no fix, the value is built elsewhere.
            if is_statement_call(node) && sprintf_value_read_once(inner, source) {
                results.push(self.diag("DSPSP", DSPSP_MESSAGE, node, None));
            }
            return;
        }
        if inner.kind() == "function_call" {
            if let Some(inner_name) = inner.child_by_field_name("name") {
                let inner_func = &source[inner_name.start_byte()..inner_name.end_byte()];
                if inner_func == "sprintf" {
                    if let Some(inner_args) = find_arguments(inner) {
                        results.push(self.diag(
                            "DSPSP",
                            DSPSP_MESSAGE,
                            node,
                            fprintf_fix(node, inner_args, source),
                        ));
                    }
                }
            }
        }
    }
}

const DSPSP_MESSAGE: &str = "'disp(sprintf(...))' can usually be replaced by 'fprintf(...\\n)'.";

/// Whether a call is a statement (its value is not used).
fn is_statement_call(call: tree_sitter::Node) -> bool {
    call.parent()
        .is_some_and(|p| matches!(p.kind(), "block" | "source_file"))
}

/// Whether the identifier `read`, the argument of `disp(read)`, is the only use of a
/// value that `name = sprintf(...)` sets, inside a function. MATLAB reports DSPSP for
/// that shape, wherever the `disp` sits in the function (probed: in an `if`, a loop,
/// a `try`, or after an unrelated statement); it is silent in a script, when the
/// value is read again (a second `disp`, `fprintf`, `t = s`, or a handle `@() s`),
/// when `name` is an input or an output, and when another definition reaches the
/// `disp`. An unknown reach is silent too.
fn sprintf_value_read_once(read: tree_sitter::Node, source: &str) -> bool {
    let name = &source[read.start_byte()..read.end_byte()];
    let Some(func) = enclosing_function_definition(read) else {
        return false;
    };
    let mut args = std::collections::HashSet::new();
    crate::analysis::control_flow::collect_function_args(func, source, &mut args);
    if args.contains(name) {
        return false;
    }
    let Some(values) = crate::analysis::reaching::reaching_values(read, source, name) else {
        return false;
    };
    let [value] = values.as_slice() else {
        return false;
    };
    if value.kind() != "function_call" {
        return false;
    }
    let Some(callee) = value.child_by_field_name("name") else {
        return false;
    };
    if &source[callee.start_byte()..callee.end_byte()] != "sprintf" {
        return false;
    }
    let mut stack = vec![func];
    while let Some(n) = stack.pop() {
        let mut cursor = n.walk();
        stack.extend(n.children(&mut cursor));
        if &source[n.start_byte()..n.end_byte()] != name {
            continue;
        }
        // A bare `name` statement is a command that displays the value: a use, which
        // stays silent here (probed: `disp(s)` then `s` on its own line is silent).
        if n.kind() == "command_name" {
            return false;
        }
        if n.kind() != "identifier"
            || is_assignment_target(n)
            || is_field_name(n)
            || n.id() == read.id()
        {
            continue;
        }
        match crate::analysis::reaching::reaching_values(n, source, name) {
            None => return false,
            Some(reached) if reached.contains(value) => return false,
            Some(_) => {}
        }
    }
    true
}

/// The nearest enclosing `function_definition` of a node.
fn enclosing_function_definition(node: tree_sitter::Node) -> Option<tree_sitter::Node> {
    let mut current = node.parent();
    while let Some(p) = current {
        if p.kind() == "function_definition" {
            return Some(p);
        }
        current = p.parent();
    }
    None
}

/// Whether an identifier is the whole target of a plain assignment (`name = ...`).
fn is_assignment_target(node: tree_sitter::Node) -> bool {
    node.parent().is_some_and(|p| {
        p.kind() == "assignment"
            && p.child_by_field_name("left")
                .is_some_and(|l| l.id() == node.id())
    })
}

/// Whether an identifier is a field name (`x.name`), not a variable.
fn is_field_name(node: tree_sitter::Node) -> bool {
    node.parent().is_some_and(|p| {
        p.kind() == "field_expression" && p.named_child(0).is_some_and(|o| o.id() != node.id())
    })
}

/// The rewrite of `disp(sprintf(fmt, ...))` to `fprintf(fmt\n, ...)`.
///
/// `disp` ends its output with a newline and `fprintf` does not, so the newline
/// goes into the format. That only works for a literal format, and only when the
/// call is a statement: as an expression `disp` returns the text it would print.
fn fprintf_fix(
    call: tree_sitter::Node,
    sprintf_args: tree_sitter::Node,
    source: &str,
) -> Option<Fix> {
    let is_statement = call
        .parent()
        .is_some_and(|p| matches!(p.kind(), "block" | "source_file"));
    let format = sprintf_args
        .named_child(0)
        .filter(|f| f.kind() == "string")?;
    let literal = &source[format.start_byte()..format.end_byte()];
    let quote = literal.chars().next().filter(|q| matches!(q, '\'' | '"'))?;
    if !is_statement || literal.len() < 2 || !literal.ends_with(quote) {
        return None;
    }
    let args_text = &source[sprintf_args.start_byte()..sprintf_args.end_byte()];
    let offset = format.end_byte() - sprintf_args.start_byte();
    // Insert `\n` in front of the closing quote of the format.
    let with_newline = format!(
        "{}\\n{}",
        &args_text[..offset - 1],
        &args_text[offset - 1..]
    );
    Some(Fix::new(
        call.start_byte()..call.end_byte(),
        format!("fprintf({with_newline})"),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_nodes};
    use malt_core::Config;

    fn engine() -> Box<dyn Rule> {
        ReadabilityEngine::from_config(&Config::default())
    }

    // -- DSPSP ---------------------------------------------------------------

    #[test]
    fn dspsp_disp_sprintf_fires() {
        let diags = lint_nodes(&*engine(), "disp(sprintf('%d', x));\n");
        assert!(has_id(&diags, "DSPSP"), "got: {diags:?}");
    }

    #[test]
    fn dspsp_disp_direct_does_not_fire() {
        let diags = lint_nodes(&*engine(), "disp(x);\n");
        assert!(!has_id(&diags, "DSPSP"), "got: {diags:?}");
    }

    #[test]
    fn dspsp_disp_of_a_sprintf_value_read_once_fires_in_a_function() {
        // Probe (dsp_02, d2a, d2n, e07, e11, e16, e21): the value reaches one `disp`.
        let src = "function f()\ns = sprintf('x');\nif true\n  disp(s)\nend\nend\n";
        assert!(has_id(&lint_nodes(&*engine(), src), "DSPSP"));
        let src = "function f()\ns = sprintf('x');\ndisp(s);\ns = 'q';\ndisp(s);\nend\n";
        let diags = lint_nodes(&*engine(), src);
        assert_eq!(
            diags.iter().filter(|d| d.rule_id == "DSPSP").count(),
            1,
            "got: {diags:?}"
        );
    }

    #[test]
    fn dspsp_disp_of_a_sprintf_value_is_silent_when_read_again_or_in_a_script() {
        // Probe (d2b, d2c, d2d, d2k, d2l, e02, e23, dsp_01, dsp_05, c04): a second use
        // (a read, a handle, a bare statement), an output, a reassignment, a script.
        for src in [
            "function f()\ns = sprintf('x');\ndisp(s);\ndisp(s);\nend\n",
            "function f()\ns = sprintf('x');\ndisp(s);\nt = s;\nend\n",
            "function f()\ns = sprintf('x');\nfprintf('%s', s);\ndisp(s);\nend\n",
            "function s = f()\ns = sprintf('x');\ndisp(s);\nend\n",
            "function f()\ns = sprintf('x');\ndisp(s);\ns\nend\n",
            "function f()\ns = sprintf('x');\nf2 = @() s;\ndisp(s);\nend\n",
            "function f()\ns = sprintf('x');\ns = 'y';\ndisp(s);\nend\n",
            "s = sprintf('x');\ndisp(s)\n",
        ] {
            let diags = lint_nodes(&*engine(), src);
            assert!(!has_id(&diags, "DSPSP"), "{src}: got: {diags:?}");
        }
    }

    /// The source after applying the DSPSP fix, or `None` when it has none.
    fn dspsp_fixed(src: &str) -> Option<String> {
        let diags = lint_nodes(&*engine(), src);
        diags.iter().find(|d| d.rule_id == "DSPSP")?.fix.as_ref()?;
        Some(malt_core::apply_fixes(src, &diags))
    }

    #[test]
    fn dspsp_fix_is_valid_matlab_with_the_newline_of_disp() {
        assert_eq!(
            dspsp_fixed("disp(sprintf('hi %d', 3));\n").as_deref(),
            Some("fprintf('hi %d\\n', 3);\n")
        );
        assert_eq!(
            dspsp_fixed("disp(sprintf(\"hi\"));\n").as_deref(),
            Some("fprintf(\"hi\\n\");\n")
        );
    }

    #[test]
    fn dspsp_has_no_fix_when_it_cannot_be_exact() {
        // Non-literal format, or `disp` used for its return value.
        assert_eq!(dspsp_fixed("disp(sprintf(fmt, 3));\n"), None);
        assert_eq!(dspsp_fixed("w = disp(sprintf('hi'));\n"), None);
        // Still reported either way.
        assert!(has_id(
            &lint_nodes(&*engine(), "disp(sprintf(fmt, 3));\n"),
            "DSPSP"
        ));
    }
}
