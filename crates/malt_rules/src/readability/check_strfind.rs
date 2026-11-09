//! # STREMP, STRCL1, STRCLFH, STRIFCND: `strfind` used as a containment test
//!
//! * STREMP: `isempty(strfind(s, p))` -> `~contains(s, p)`.
//! * STRCL1: `cellfun('isempty', strfind(s, p))` -> `~contains(s, p)`.
//! * STRCLFH: `cellfun(@isempty, strfind(s, p))` -> `~contains(s, p)`.
//! * STRIFCND: `if strfind(s, p)` -> `if contains(s, p)`.

use super::*;

/// The (text, text) arguments of a two-argument `strfind(s, p)` call.
fn strfind_operands<'a>(n: tree_sitter::Node, source: &'a str) -> Option<(&'a str, &'a str)> {
    if n.kind() != "function_call" || call_name(n, source) != Some("strfind") {
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

/// `~contains(s, p)`, parenthesised unless `node` already sits where a unary
/// expression is unambiguous (statement, assignment value, argument, condition).
fn not_contains(node: tree_sitter::Node, s: &str, p: &str) -> String {
    let bare = node.parent().is_none_or(|parent| {
        matches!(
            parent.kind(),
            "assignment"
                | "arguments"
                | "if_statement"
                | "elseif_clause"
                | "while_statement"
                | "parenthesized_expression"
                | "block"
                | "source_file"
        )
    });
    if bare {
        format!("~contains({s}, {p})")
    } else {
        format!("(~contains({s}, {p}))")
    }
}

impl ReadabilityEngine {
    /// STREMP: `isempty(strfind(s, p))`.
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
        if let Some((s, p)) = strfind_operands(args[0], source) {
            results.push(self.diag(
                "STREMP",
                "For readability, use '~contains(str1, str2)' instead of 'isempty(strfind(str1, str2))'.",
                node,
                Some(Fix::new(
                    node.start_byte()..node.end_byte(),
                    not_contains(node, s, p),
                )),
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
            Some(Fix::new(
                node.start_byte()..node.end_byte(),
                not_contains(node, s, p),
            )),
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
        let Some(parent) = node.parent() else {
            return;
        };
        let is_condition = matches!(
            parent.kind(),
            "if_statement" | "elseif_clause" | "while_statement"
        ) && parent
            .child_by_field_name("condition")
            .is_some_and(|c| c.id() == node.id());
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
