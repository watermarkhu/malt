//! # ISCHR, ISSTR, ISLOG, ISCEL: comparing the class to a type name
//!
//! `check_class_compare` flags `strcmp(class(x), 'char')` and
//! `isequal(class(x), 'char')` (and the same for `'struct'`, `'logical'`,
//! `'cell'`) and suggests the dedicated predicate (`ischar`, `isstruct`,
//! `islogical`, `iscell`).

use super::*;

/// Class name -> (check id, predicate, message).
const CLASS_CHECKS: &[(&str, &str, &str, &str)] = &[
    (
        "char",
        "ISCHR",
        "ischar",
        "Use ISCHAR instead of comparing the class to 'char'.",
    ),
    (
        "struct",
        "ISSTR",
        "isstruct",
        "Use ISSTRUCT instead of comparing the class to 'struct'.",
    ),
    (
        "logical",
        "ISLOG",
        "islogical",
        "Use ISLOGICAL instead of comparing the class to 'logical'.",
    ),
    (
        "cell",
        "ISCEL",
        "iscell",
        "Use ISCELL instead of comparing the class to 'cell'.",
    ),
];

/// The argument text of `class(x)`, or `None` when `n` is not such a call.
fn class_operand<'a>(n: tree_sitter::Node, source: &'a str) -> Option<&'a str> {
    if n.kind() != "function_call" || call_name(n, source) != Some("class") {
        return None;
    }
    let args = call_args(n);
    (args.len() == 1).then(|| &source[args[0].start_byte()..args[0].end_byte()])
}

/// The text between the quotes of a string literal node.
fn string_value<'a>(n: tree_sitter::Node, source: &'a str) -> Option<&'a str> {
    if n.kind() != "string" {
        return None;
    }
    let t = &source[n.start_byte()..n.end_byte()];
    (t.len() >= 2).then(|| &t[1..t.len() - 1])
}

impl ReadabilityEngine {
    /// `strcmp(class(x), 'char')` / `isequal(class(x), 'char')`.
    pub(crate) fn check_class_compare(
        &self,
        node: tree_sitter::Node,
        source: &str,
        results: &mut Vec<Diagnostic>,
    ) {
        let args = call_args(node);
        if args.len() != 2 {
            return;
        }
        for (call, lit) in [(args[0], args[1]), (args[1], args[0])] {
            let (Some(operand), Some(ty)) =
                (class_operand(call, source), string_value(lit, source))
            else {
                continue;
            };
            let Some(&(_, id, pred, message)) = CLASS_CHECKS.iter().find(|(c, ..)| *c == ty) else {
                continue;
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
                    format!("{pred}({operand})"),
                )),
            ));
            return;
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
    fn strcmp_class_fires_for_each_type() {
        for (ty, id) in [
            ("char", "ISCHR"),
            ("struct", "ISSTR"),
            ("logical", "ISLOG"),
            ("cell", "ISCEL"),
        ] {
            let diags = lint_nodes(&*engine(), &format!("x = strcmp(class(a), '{ty}');\n"));
            assert!(has_id(&diags, id), "{ty}: got {diags:?}");
        }
    }

    #[test]
    fn reversed_and_isequal_forms_fire() {
        for src in [
            "x = strcmp('char', class(a));\n",
            "x = isequal(class(a), 'char');\n",
        ] {
            assert!(has_id(&lint_nodes(&*engine(), src), "ISCHR"), "{src}");
        }
    }

    #[test]
    fn isa_and_other_classes_do_not_fire() {
        for src in [
            "x = isa(a, 'char');\n",
            "x = isa(a, 'double');\n",
            "x = isa(a, 'string');\n",
            "x = strcmp(class(a), 'double');\n",
            "x = strcmp(a, 'char');\n",
        ] {
            let diags = lint_nodes(&*engine(), src);
            assert!(
                !diags
                    .iter()
                    .any(|d| ["ISCHR", "ISSTR", "ISLOG", "ISCEL", "ISMAT"].contains(&d.rule_id)),
                "{src}: got {diags:?}"
            );
        }
    }

    #[test]
    fn fix_uses_the_predicate() {
        let src = "x = strcmp(class(a.b), 'struct');\n";
        let diags = lint_nodes(&*engine(), src);
        assert_eq!(malt_core::apply_fixes(src, &diags), "x = isstruct(a.b);\n");
    }
}
