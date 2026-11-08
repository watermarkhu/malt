use super::*;

impl GoodPracticesEngine {
    /// ATTF / ATTOF: class-level `Abstract` attribute.
    ///
    /// ATTF fires when the value assigned to `Abstract` is not a recognizable
    /// boolean literal; ATTOF fires when the value is explicitly `false`.
    pub(crate) fn check_attf_attof(
        &self,
        tree: &tree_sitter::Tree,
        source: &str,
    ) -> Vec<Diagnostic> {
        if !self.is_check_enabled("ATTF") && !self.is_check_enabled("ATTOF") {
            return Vec::new();
        }

        let meta = FileMeta::build(tree, source);
        let Some(class) = meta.class.as_ref() else {
            return Vec::new();
        };

        let attr = match class
            .attributes
            .iter()
            .find(|a| a.name == "Abstract" && !a.negated)
        {
            Some(a) => a,
            None => return Vec::new(),
        };

        let Some(value) = attr.value.as_deref() else {
            return Vec::new();
        };

        let class_node = find_child_of_kind(tree.root_node(), "class_definition");
        let attr_node = class_node.and_then(|cn| find_attribute_node(cn, "Abstract", source));
        let (start_byte, end_byte, line, column) = match attr_node {
            Some(an) => (
                an.start_byte(),
                an.end_byte(),
                an.start_position().row + 1,
                an.start_position().column + 1,
            ),
            None => (class.byte_range.start, class.byte_range.end, class.line, 1),
        };

        let lower = value.to_lowercase();
        let is_valid_literal =
            matches!(lower.as_str(), "true" | "false" | "1" | "0" | "on" | "off");

        let mut diagnostics = Vec::new();
        if !is_valid_literal && self.is_check_enabled("ATTF") {
            diagnostics.push(Diagnostic {
                rule_id: "ATTF",
                message: "The Code Analyzer is unable to determine if the expression assigned to the Abstract attribute evaluates to true or false.".to_string(),
                severity: Severity::Warning,
                byte_range: start_byte..end_byte,
                line,
                column,
                fix: None,
            });
        } else if lower == "false" && self.is_check_enabled("ATTOF") {
            diagnostics.push(Diagnostic {
                rule_id: "ATTOF",
                message: "Setting the class attribute Abstract to false is not recommended."
                    .to_string(),
                severity: Severity::Info,
                byte_range: start_byte..end_byte,
                line,
                column,
                fix: None,
            });
        }
        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attof_fires_on_abstract_false() {
        let source = "classdef (Abstract = false) Foo\nend\n";
        let tree = parse(source);
        let eng = engine();
        let diags = eng.check_attf_attof(&tree, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "ATTOF");
    }

    #[test]
    fn test_attf_fires_on_non_literal_abstract() {
        let source = "classdef (Abstract = someVar) Bar\nend\n";
        let tree = parse(source);
        let eng = engine();
        let diags = eng.check_attf_attof(&tree, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "ATTF");
    }

    #[test]
    fn test_attf_attof_silent_on_abstract_true() {
        let source = "classdef (Abstract = true) Foo\nend\n";
        let tree = parse(source);
        let eng = engine();
        assert!(eng.check_attf_attof(&tree, source).is_empty());
    }

    #[test]
    fn test_attf_attof_respect_disabled_checks() {
        let eng = GoodPracticesEngine {
            config: GoodPracticesConfig {
                max_variable_name_length: 63,
                disabled_checks: vec!["ATTF".to_string(), "ATTOF".to_string()],
            },
        };
        let source = "classdef (Abstract = false) Foo\nend\n";
        let tree = parse(source);
        assert!(eng.check_attf_attof(&tree, source).is_empty());
    }
}
