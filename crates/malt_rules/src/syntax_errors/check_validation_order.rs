//! `check_validation_order`: one check of the Syntax Errors engine.

use super::*;

/// MATLAB quotes punctuation in these messages (`'('`) but not names (`double`).
fn quoted(token: &str) -> String {
    if token.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        token.to_string()
    } else {
        format!("'{token}'")
    }
}

impl SyntaxErrorsEngine {
    /// Validation order in `arguments` and `properties` declarations. Probed against
    /// R2026b: after the first `{...}` validator group, a size `(`, a second group or
    /// a class name is out of order. MATLAB reports the first such token only.
    pub(crate) fn check_validation_order(&self, root: Node, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        self.walk_validation_order(root, source, &mut diagnostics);
        diagnostics
    }

    pub(crate) fn walk_validation_order(
        &self,
        node: Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if node.kind() == "property" {
            self.check_property_validation_order(node, source, diagnostics);
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.walk_validation_order(child, source, diagnostics);
        }
    }

    /// One `property` declaration. In a `properties` block the out-of-order token is
    /// MCPLD (probed: `x {mustBeReal} double` reports `double`); in an `arguments`
    /// block it is FVSYN. A property default that indexes a literal is MCPLD too.
    pub(crate) fn check_property_validation_order(
        &self,
        property: Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let in_properties = Self::ancestor_of_kind(property, "properties").is_some()
            && Self::ancestor_of_kind(property, "arguments").is_none();
        if in_properties {
            self.check_property_index(property, source, diagnostics);
        }
        // VTPOD: the class written before the size (`x double (1,1)`), in an
        // `arguments` block and in a `properties` block alike (probed, plan 6.2:
        // reported at the class name, with or without validators and a default).
        if let Some(class) = Self::class_before_size(property) {
            if self.is_check_enabled("VTPOD") {
                let start = class.start_byte();
                let before = &source[..start];
                diagnostics.push(Diagnostic {
                    rule_id: "VTPOD",
                    message:
                        "Specify validation in the following order: size, then class, then functions."
                            .to_string(),
                    severity: Severity::Error,
                    byte_range: start..class.end_byte(),
                    line: before.matches('\n').count() + 1,
                    column: start - before.rfind('\n').map_or(0, |newline| newline + 1) + 1,
                    fix: None,
                });
            }
            return;
        }
        let Some(vf) = Self::find_validation_functions(property) else {
            return;
        };
        let Some((start, end)) = Self::token_after_validators(vf, source) else {
            return;
        };
        let (rule_id, message) = if in_properties {
            (
                "MCPLD",
                format!(
                    "Invalid property syntax at {}.",
                    quoted(&source[start..end])
                ),
            )
        } else {
            (
                "FVSYN",
                format!(
                    "Invalid function argument syntax at {}.",
                    quoted(&source[start..end])
                ),
            )
        };
        if !self.is_check_enabled(rule_id) {
            return;
        }
        let before = &source[..start];
        diagnostics.push(Diagnostic {
            rule_id,
            message,
            severity: Severity::Error,
            byte_range: start..end,
            line: before.matches('\n').count() + 1,
            column: start - before.rfind('\n').map_or(0, |newline| newline + 1) + 1,
            fix: None,
        });
    }

    /// MCPLD for a property default that indexes a literal: `e = [1 2 3](2)` is
    /// reported at the `(` (probed). MATLAB reports nothing else for the `end` in it.
    fn check_property_index(
        &self,
        property: Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("MCPLD") {
            return;
        }
        let Some(start) = Self::literal_index_paren(property, source) else {
            return;
        };
        let before = &source[..start];
        diagnostics.push(Diagnostic {
            rule_id: "MCPLD",
            message: "Invalid property syntax at '('.".to_string(),
            severity: Severity::Error,
            byte_range: start..start + 1,
            line: before.matches('\n').count() + 1,
            column: start - before.rfind('\n').map_or(0, |newline| newline + 1) + 1,
            fix: None,
        });
    }

    /// Byte offset of the `(` in a property default that indexes a literal.
    fn literal_index_paren(property: Node, source: &str) -> Option<usize> {
        let text = &source[property.start_byte()..property.end_byte()];
        let equals = text.find('=')?;
        let default = &text[equals..];
        let index = default.find("](").or_else(|| default.find("}("))?;
        Some(property.start_byte() + equals + index + 1)
    }

    /// The class name of a declaration that has it before the size: the parser
    /// leaves it as an ERROR node holding one identifier, right before `dimensions`.
    fn class_before_size(property: Node) -> Option<Node> {
        let mut cursor = property.walk();
        let children: Vec<Node> = property.named_children(&mut cursor).collect();
        children.windows(2).find_map(|pair| {
            let [error, next] = pair else {
                return None;
            };
            (error.kind() == "ERROR"
                && next.kind() == "dimensions"
                && error.named_child_count() == 1)
                .then(|| error.named_child(0))
                .flatten()
                .filter(|id| id.kind() == "identifier")
        })
    }

    /// True when this check reports the property's out-of-order token or literal
    /// index, so the generic ERROR-node MCPLD must not repeat it.
    pub(crate) fn property_order_reported(property: Node, source: &str) -> bool {
        Self::class_before_size(property).is_some()
            || Self::find_validation_functions(property)
                .and_then(|vf| Self::token_after_validators(vf, source))
                .is_some()
            || Self::literal_index_paren(property, source).is_some()
    }

    /// The first token after the validator group `vf` that MATLAB rejects: a size
    /// `(`, a second `{` group, or a class name, on the same line as the group.
    /// Returns its byte range.
    fn token_after_validators(vf: Node, source: &str) -> Option<(usize, usize)> {
        let open = vf.start_byte() + source[vf.start_byte()..vf.end_byte()].find('{')?;
        let mut depth = 0usize;
        let mut close = None;
        for (offset, c) in source[open..].char_indices() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        close = Some(open + offset);
                        break;
                    }
                }
                _ => {}
            }
        }
        let rest = &source[close? + 1..];
        let trimmed = rest.trim_start_matches([' ', '\t']);
        let start = source.len() - trimmed.len();
        let first = trimmed.chars().next()?;
        if first == '{' || first == '(' {
            return Some((start, start + 1));
        }
        if first.is_ascii_alphabetic() || first == '_' {
            let len = trimmed
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .unwrap_or(trimmed.len());
            return Some((start, start + len));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_file};
    use malt_core::Config;

    fn engine() -> Box<dyn Rule> {
        SyntaxErrorsEngine::from_config(&Config::default())
    }

    fn engine_with_disabled(checks: &[&str]) -> Box<dyn Rule> {
        let disabled = checks
            .iter()
            .map(|c| format!("\"{c}\""))
            .collect::<Vec<_>>()
            .join(", ");
        let config = Config::from_toml(&format!(
            "[lint.rules.SYNTAX_ERRORS_ENGINE]\ndisabled_checks = [{disabled}]\n"
        ))
        .unwrap();
        SyntaxErrorsEngine::from_config(&config)
    }

    /// The (line, message) of each diagnostic with `id`.
    fn found(src: &str, id: &str) -> Vec<(usize, String)> {
        lint_file(&*engine(), src)
            .into_iter()
            .filter(|d| d.rule_id == id)
            .map(|d| (d.line, d.message))
            .collect()
    }

    // -- FVSYN: out-of-order tokens after the validator group ---------------

    #[test]
    fn fvsyn_fires_on_class_after_validation_functions() {
        let src = "function f(x)\n    arguments\n        x {mustBeReal} double\n    end\nend\n";
        let diags = lint_file(&*engine(), src);
        assert!(has_id(&diags, "FVSYN"), "got: {diags:?}");
        assert!(!has_id(&diags, "VTPOD"), "got: {diags:?}");
    }

    #[test]
    fn fvsyn_fires_on_class_after_validation_functions_with_dims() {
        let src =
            "function f(x)\n    arguments\n        x (1,1) {mustBeReal} double\n    end\nend\n";
        let diags = lint_file(&*engine(), src);
        assert!(has_id(&diags, "FVSYN"), "got: {diags:?}");
    }

    #[test]
    fn fvsyn_fires_on_size_after_validation_functions() {
        // Probed (vo_vf_first.m): the `(` is reported, quoted.
        let src =
            "function f(x)\n    arguments\n        x {mustBeReal} (1,1) double\n    end\nend\n";
        assert_eq!(
            found(src, "FVSYN"),
            vec![(3, "Invalid function argument syntax at '('.".to_string())]
        );
    }

    #[test]
    fn fvsyn_fires_on_second_validator_group() {
        // Probed (vo_two.m): the second `{` is reported.
        let src = "function f(x)\n    arguments\n        x {mustBeReal} {mustBePositive} double\n    end\nend\n";
        assert_eq!(
            found(src, "FVSYN"),
            vec![(3, "Invalid function argument syntax at '{'.".to_string())]
        );
    }

    #[test]
    fn fvsyn_fires_on_class_after_default_free_validators() {
        // Probed (vo_default.m, vo_b1.m): a class after the group, before `= 1`.
        let src = "function f(x)\n    arguments\n        x {mustBeReal} double = 1\n    end\nend\n";
        assert_eq!(
            found(src, "FVSYN"),
            vec![(3, "Invalid function argument syntax at double.".to_string())]
        );
    }

    #[test]
    fn fvsyn_ok_on_class_inside_braces() {
        // MATLAB accepts a class name among the validators (probe: no finding).
        let src =
            "function f(x)\n    arguments\n        x (1,1) {mustBeReal, double}\n    end\nend\n";
        let diags = lint_file(&*engine(), src);
        assert!(!has_id(&diags, "FVSYN"), "got: {diags:?}");
    }

    #[test]
    fn fvsyn_ok_on_sole_class_validator() {
        let src = "function f(x)\n    arguments\n        x {double}\n    end\nend\n";
        let diags = lint_file(&*engine(), src);
        assert!(!has_id(&diags, "FVSYN"), "got: {diags:?}");
    }

    #[test]
    fn fvsyn_ok_on_correct_order() {
        let src = "function f(x)\n    arguments\n        x (1,1) double {mustBeReal}\n        y (1,1) double\n        z {mustBePositive}\n        w (1,1) double {mustBeReal} = 1\n    end\nend\n";
        let diags = lint_file(&*engine(), src);
        assert!(!has_id(&diags, "FVSYN"), "got: {diags:?}");
    }

    #[test]
    fn fvsyn_ok_on_class_without_dims() {
        let src = "function f(x)\n    arguments\n        x double {mustBePositive}\n    end\nend\n";
        let diags = lint_file(&*engine(), src);
        assert!(!has_id(&diags, "FVSYN"), "got: {diags:?}");
    }

    #[test]
    fn fvsyn_disabled_via_config() {
        let engine = engine_with_disabled(&["FVSYN"]);
        let src = "function f(x)\n    arguments\n        x {mustBeReal} double\n    end\nend\n";
        let diags = lint_file(&*engine, src);
        assert!(!has_id(&diags, "FVSYN"), "got: {diags:?}");
    }

    // -- MCPLD: out-of-order tokens in a properties block -------------------

    #[test]
    fn mcpld_fires_on_class_after_validators_in_properties() {
        // Probed (ValCls.m): MATLAB reports MCPLD at `double` (line 3), not FVSYN.
        let src = "classdef ValCls\n    properties\n        x {mustBeReal} double\n    end\nend\n";
        assert_eq!(
            found(src, "MCPLD"),
            vec![(3, "Invalid property syntax at double.".to_string())]
        );
        assert!(!has_id(&lint_file(&*engine(), src), "FVSYN"));
    }

    #[test]
    fn mcpld_fires_on_size_after_validators_in_properties() {
        // Probed (PropC.m): `x (1,1) {mustBeReal} double` reports the class.
        let src =
            "classdef PropC\n    properties\n        x (1,1) {mustBeReal} double\n    end\nend\n";
        assert_eq!(
            found(src, "MCPLD"),
            vec![(3, "Invalid property syntax at double.".to_string())]
        );
    }

    #[test]
    fn mcpld_fires_on_index_of_literal_in_properties() {
        // Probed (PropF.m, PropIdx.m): the `(` of `[1 2 3](2)` is MCPLD at line 3.
        let src = "classdef PropF\n    properties\n        e = [1 2 3](2)\n    end\nend\n";
        assert_eq!(
            found(src, "MCPLD"),
            vec![(3, "Invalid property syntax at '('.".to_string())]
        );
        let end_src = "classdef PropIdx\n    properties\n        e = [1 2 3](end)\n    end\nend\n";
        assert_eq!(found(end_src, "MCPLD").len(), 1);
    }

    #[test]
    fn vtpod_fires_on_class_before_size() {
        // Probed (plan 6.2): reported at the class name, in `arguments` and `properties`
        // blocks, with or without validators and a default; no MCPLD or FVSYN there.
        for src in [
            "function f(a)\n    arguments\n        a double (1,1)\n    end\n    disp(a);\nend\n",
            "function f(a)\n    arguments\n        a double (1,1) {mustBePositive} = 1\n    end\n    disp(a);\nend\n",
            "classdef C\n    properties\n        P double (1,1) = 1\n    end\nend\n",
        ] {
            let diags = lint_file(&*engine(), src);
            assert_eq!(found(src, "VTPOD").len(), 1, "{src}: {diags:?}");
            assert_eq!(found(src, "VTPOD")[0].0, 3, "{src}");
            assert!(!has_id(&diags, "MCPLD") && !has_id(&diags, "FVSYN"), "{src}: {diags:?}");
        }
        let ok = "function f(a)\n    arguments\n        a (1,1) double {mustBePositive} = 1\n    end\n    disp(a);\nend\n";
        assert!(found(ok, "VTPOD").is_empty());
    }

    #[test]
    fn mcpld_ok_on_valid_property_and_plain_default() {
        // Probed (PropD.m, PropD2): size, class and validators in order, a plain default.
        for src in [
            "classdef PropD\n    properties\n        x (1,1) double {mustBeReal} = 1\n    end\nend\n",
            "classdef PropE\n    properties\n        e = [1 2 3]\n    end\nend\n",
            "classdef PropG\n    properties\n        x double {mustBePositive}\n    end\nend\n",
        ] {
            assert!(found(src, "MCPLD").is_empty(), "{src:?}");
        }
    }

    #[test]
    fn mcpld_not_affected_by_disabled_fvsyn() {
        let src = "classdef ValCls\n    properties\n        x {mustBeReal} double\n    end\nend\n";
        let diags = lint_file(&*engine_with_disabled(&["FVSYN"]), src);
        assert!(has_id(&diags, "MCPLD"), "got: {diags:?}");
    }
}
