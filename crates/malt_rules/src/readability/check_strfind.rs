//! # STREMP, STRCL1, STRCLFH, STRIFCND: `strfind` used as a containment test
//!
//! * STREMP: `isempty(strfind(s, p))` -> `~contains(s, p)`.
//! * STRCL1: `cellfun('isempty', strfind(s, p))` -> `~contains(s, p)`.
//! * STRCLFH: `cellfun(@isempty, strfind(s, p))` -> `~contains(s, p)`.
//! * STRIFCND: `if strfind(s, p)` -> `if contains(s, p)`.

use super::*;

/// The verbatim argument text of a two-argument `strfind(s, p)` call (`a,b`),
/// with the spacing of the source. MATLAB's fix reprints the arguments this way.
fn strfind_arg_text<'a>(n: tree_sitter::Node, source: &'a str) -> Option<&'a str> {
    if n.kind() != "function_call" || call_path(n, source, Dotted::Skip) != Some("strfind") {
        return None;
    }
    let args = call_args(n);
    (args.len() == 2).then(|| &source[args[0].start_byte()..args[1].end_byte()])
}

/// The (text, text) arguments of a two-argument `strfind(s, p)` call.
fn strfind_operands<'a>(n: tree_sitter::Node, source: &'a str) -> Option<(&'a str, &'a str)> {
    if n.kind() != "function_call" || call_path(n, source, Dotted::Skip) != Some("strfind") {
        return None;
    }
    let args = call_args(n);
    (args.len() == 2).then(|| {
        (
            &source[args[0].start_byte()..args[0].end_byte()],
            &source[args[1].start_byte()..args[1].end_byte()],
        )
    })
}

/// `~` in front of `replacement`, parenthesised unless `node` already sits where
/// a unary expression is unambiguous (statement, assignment value, argument,
/// condition).
fn not_contains(node: tree_sitter::Node, replacement: &str) -> Fix {
    let bare = node.parent().is_none_or(|parent| {
        matches!(
            parent.kind(),
            "assignment"
                | "arguments"
                | "if_statement"
                | "elseif_clause"
                | "while_statement"
                | "parenthesis"
                | "block"
                | "source_file"
        )
    });
    if bare {
        Fix::new(
            node.start_byte()..node.end_byte(),
            format!("~{replacement}"),
        )
    } else {
        Fix::new(
            node.start_byte()..node.end_byte(),
            format!("(~{replacement})"),
        )
    }
}

impl ReadabilityEngine {
    /// STREMP: `isempty(strfind(s, p))`, negated or not.
    ///
    /// MATLAB's own fix rewrites the whole negated form: `~isempty(strfind(a,b))`
    /// becomes `contains(a,b)` (probed), not `~(~contains(a,b))`. The arguments
    /// are reprinted verbatim, with the spacing the source had.
    pub(crate) fn check_isempty_strfind(
        &self,
        node: tree_sitter::Node,
        source: &str,
        results: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("STREMP") {
            return;
        }
        let args = call_args(node);
        if args.len() != 1 {
            return;
        }
        if let Some(arg_text) = strfind_arg_text(args[0], source) {
            let negated = node.parent().is_some_and(|parent| {
                parent.kind() == "not_operator"
                    && parent
                        .named_child(0)
                        .is_some_and(|operand| operand.id() == node.id())
            });
            let fix = if negated {
                // The `~` in front is part of the rewrite: span the not_operator.
                let outer = node.parent().expect("checked above");
                Fix::new(
                    outer.start_byte()..outer.end_byte(),
                    format!("contains({arg_text})"),
                )
            } else {
                not_contains(node, &format!("contains({arg_text})"))
            };
            results.push(self.diag(
                "STREMP",
                "For readability, use '~contains(str1, str2)' instead of 'isempty(strfind(str1, str2))'.",
                node,
                Some(fix),
            ));
        }
    }

    /// STRCL1 / STRCLFH: `cellfun('isempty', strfind(s, p))` and
    /// `cellfun(@isempty, strfind(s, p))`.
    pub(crate) fn check_cellfun_strfind(
        &self,
        node: tree_sitter::Node,
        source: &str,
        results: &mut Vec<Diagnostic>,
    ) {
        let args = call_args(node);
        if args.len() != 2 {
            return;
        }
        let Some((s, p)) = strfind_operands(args[1], source) else {
            return;
        };
        let (id, message) = match &source[args[0].start_byte()..args[0].end_byte()] {
            "'isempty'" | "\"isempty\"" => (
                "STRCL1",
                "For readability, use '~contains(str1, str2)' instead of 'cellfun('isempty', strfind(str1, str2))'.",
            ),
            "@isempty" => (
                "STRCLFH",
                "For readability, use '~contains(str1, str2)' instead of 'cellfun(@isempty, strfind(str1, str2))'.",
            ),
            _ => return,
        };
        if !self.is_check_enabled(id) {
            return;
        }
        results.push(self.diag(
            id,
            message,
            node,
            // MATLAB offers no fix for the cellfun forms (probed: manual), so the
            // argument spacing here is malt's own.
            Some(not_contains(node, &format!("contains({s}, {p})"))),
        ));
    }

    /// STRIFCND: `strfind(s, p)` used directly as an `if`/`while` condition.
    pub(crate) fn check_strfind_condition(
        &self,
        node: tree_sitter::Node,
        source: &str,
        results: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("STRIFCND") || strfind_operands(node, source).is_none() {
            return;
        }
        // `if (strfind(s, p))`: a parenthesis sits between the call and the condition.
        let mut outer = node;
        while let Some(parent) = outer.parent().filter(|p| p.kind() == "parenthesis") {
            outer = parent;
        }
        let Some(parent) = outer.parent() else {
            return;
        };
        let is_condition = matches!(
            parent.kind(),
            "if_statement" | "elseif_clause" | "while_statement"
        ) && parent
            .child_by_field_name("condition")
            .is_some_and(|c| c.id() == outer.id());
        if is_condition {
            results.push(self.diag(
                "STRIFCND",
                "For readability, use 'contains(str1, str2)' instead of 'strfind(str1, str2)'.",
                node,
                Some(Fix::new(node.start_byte()..node.end_byte(), {
                    let t = &source[node.start_byte()..node.end_byte()];
                    t.replacen("strfind", "contains", 1)
                })),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_nodes};
    use malt_core::Config;

    #[test]
    fn strifcnd_fires_through_parentheses_in_a_condition() {
        // Probed: MATLAB reports STRIFCND for `if (strfind(s, 'a'))` on line 2, and
        // reports neither STREMP nor STRIFCND for `isempty((strfind(s, 'a')))`.
        let engine = ReadabilityEngine::from_config(&Config::default());
        let diags = lint_nodes(
            &*engine,
            "s = 'abc';\nif (strfind(s, 'a'))\n  disp(1)\nend\n",
        );
        assert!(
            diags.iter().any(|d| d.rule_id == "STRIFCND" && d.line == 2),
            "{diags:?}"
        );
        let diags = lint_nodes(&*engine, "s = 'abc';\nb = isempty((strfind(s, 'a')));\n");
        assert!(!has_id(&diags, "STREMP"), "{diags:?}");
    }

    fn engine() -> Box<dyn Rule> {
        ReadabilityEngine::from_config(&Config::default())
    }

    #[test]
    fn stremp_isempty_strfind_fires_with_unsafe_fix() {
        let src = "x = isempty(strfind(s, 'a'));\n";
        let diags = lint_nodes(&*engine(), src);
        let d = diags
            .iter()
            .find(|d| d.rule_id == "STREMP")
            .expect("STREMP");
        assert!(!d.fix.as_ref().expect("fix").is_safe());
        let (fixed, _) = malt_core::apply_fixes_with(src, &diags, malt_core::FixMode::All);
        assert_eq!(fixed, "x = ~contains(s, 'a');\n");
    }

    #[test]
    fn stremp_negated_form_rewrites_the_whole_negation_as_matlab_does() {
        // Probed: MATLAB's fix on `t = ~isempty(strfind(a,b));` produces
        // `t = contains(a,b);` — the `~` is consumed, not doubled.
        let src = "t = ~isempty(strfind(a,b));\n";
        let diags = lint_nodes(&*engine(), src);
        let d = diags
            .iter()
            .find(|d| d.rule_id == "STREMP")
            .expect("STREMP");
        let fix = d.fix.as_ref().expect("fix");
        assert!(!fix.is_safe());
        let mut fixed = String::from(src);
        fixed.replace_range(fix.byte_range.clone(), &fix.replacement);
        assert_eq!(fixed, "t = contains(a,b);\n");
    }

    #[test]
    fn stremp_keeps_the_source_argument_spacing() {
        // Probed: MATLAB reprints the strfind arguments verbatim (`a,b` stays
        // `a,b`, not `a, b`).
        let src = "x = isempty(strfind(a , 'a'));\n";
        let diags = lint_nodes(&*engine(), src);
        let d = diags
            .iter()
            .find(|d| d.rule_id == "STREMP")
            .expect("STREMP");
        let fix = d.fix.as_ref().expect("fix");
        let mut fixed = String::from(src);
        fixed.replace_range(fix.byte_range.clone(), &fix.replacement);
        assert_eq!(fixed, "x = ~contains(a , 'a');\n");
    }

    #[test]
    fn stremp_other_isempty_and_strcmp_do_not_fire() {
        for src in [
            "x = isempty(s);\n",
            "x = isempty(find(s));\n",
            "x = strcmp(s, '');\n",
            "x = isempty(strfind(s));\n",
        ] {
            assert!(!has_id(&lint_nodes(&*engine(), src), "STREMP"), "{src}");
        }
    }

    #[test]
    fn strcl_cellfun_forms_fire() {
        let diags = lint_nodes(&*engine(), "x = cellfun('isempty', strfind(c, 'a'));\n");
        assert!(has_id(&diags, "STRCL1"), "got: {diags:?}");
        let diags = lint_nodes(&*engine(), "x = cellfun(@isempty, strfind(c, 'a'));\n");
        assert!(has_id(&diags, "STRCLFH"), "got: {diags:?}");
    }

    #[test]
    fn strcl_other_cellfun_and_bare_strfind_do_not_fire() {
        for src in [
            "x = cellfun(@numel, strfind(c, 'a'));\n",
            "x = cellfun(@isempty, c);\n",
            "x = strfind(c, 'a');\n",
            "x = strncmp(a, b, 3);\n",
        ] {
            let diags = lint_nodes(&*engine(), src);
            assert!(
                !has_id(&diags, "STRCL1") && !has_id(&diags, "STRCLFH"),
                "{src}: {diags:?}"
            );
        }
    }

    #[test]
    fn strifcnd_strfind_as_condition_fires() {
        for src in [
            "if strfind(a, 'b')\n    x = 1;\nend\n",
            "while strfind(a, 'b')\n    x = 1;\nend\n",
        ] {
            assert!(has_id(&lint_nodes(&*engine(), src), "STRIFCND"), "{src}");
        }
    }

    #[test]
    fn strifcnd_other_uses_do_not_fire() {
        for src in [
            "x = strfind(a, 'b');\n",
            "if ~isempty(strfind(a, 'b'))\n    x = 1;\nend\n",
            "if strcmp(a, b)\n    x = 1;\nend\n",
            "if strfind(a, 'b') > 3\n    x = 1;\nend\n",
        ] {
            assert!(!has_id(&lint_nodes(&*engine(), src), "STRIFCND"), "{src}");
        }
    }
}
