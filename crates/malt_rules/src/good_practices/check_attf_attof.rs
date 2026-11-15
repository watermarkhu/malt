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

        // The named attributes drop the unnamed `~Abstract` form, so it is read from the
        // written ones.
        let class_node = find_child_of_kind(tree.root_node(), "class_definition");
        let written = class_node
            .map(|cn| attributes_of(cn, source, AttrForms::Written))
            .unwrap_or_default();
        if let Some(neg) = written.iter().find(|a| {
            a.negated && a.name == "Abstract" && &source[a.byte_range.clone()] == "~Abstract"
        }) {
            if !self.is_check_enabled("ATTOF") {
                return Vec::new();
            }
            return vec![Diagnostic {
                rule_id: "ATTOF",
                message: "Setting the class attribute Abstract to false is not recommended."
                    .to_string(),
                severity: Severity::Warning,
                byte_range: neg.byte_range.clone(),
                line: neg.line,
                column: neg.column,
                fix: None,
            }];
        }
        let attr = match class.attributes.iter().find(|a| a.name == "Abstract") {
            Some(a) => a,
            None => return Vec::new(),
        };
        // `~Abstract` is `Abstract = false`; a bare `Abstract` is true and not checked.
        let value = match attr.value.as_deref() {
            _ if attr.negated => "false",
            Some(v) => v,
            None => return Vec::new(),
        };

        let (start_byte, end_byte, line, column) = (
            attr.byte_range.start,
            attr.byte_range.end,
            attr.line,
            attr.column,
        );

        // MATLAB reads the literals case-sensitively: `Abstract = FALSE` is ATTF (probed).
        let is_valid_literal = matches!(value, "true" | "false" | "1" | "0" | "on" | "off");

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
        } else if matches!(value, "false" | "0") && self.is_check_enabled("ATTOF") {
            diagnostics.push(Diagnostic {
                rule_id: "ATTOF",
                message: "Setting the class attribute Abstract to false is not recommended."
                    .to_string(),
                severity: Severity::Warning,
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

#[cfg(test)]
mod probe_p2a_tests {
    use crate::test_util::lint_file;
    use malt_core::Config;

    /// Lines of the `id` diagnostics that the default (mathworks) engine reports for `src`.
    fn lines(src: &str, id: &str) -> Vec<usize> {
        let e = crate::good_practices::GoodPracticesEngine::from_config(&Config::default());
        lint_file(&*e, src)
            .iter()
            .filter(|d| d.rule_id == id)
            .map(|d| d.line)
            .collect()
    }

    #[test]
    fn attf_and_attof_follow_the_probed_literals() {
        // Probed: `FALSE` is not understood (ATTF); `0` and `~Abstract` are false (ATTOF).
        assert_eq!(
            lines("classdef (Abstract = FALSE) C\nend\n", "ATTF"),
            vec![1]
        );
        assert_eq!(lines("classdef (Abstract = 0) C\nend\n", "ATTOF"), vec![1]);
        assert_eq!(lines("classdef (~Abstract) C\nend\n", "ATTOF"), vec![1]);
        assert!(lines("classdef (Abstract = true) C\nend\n", "ATTOF").is_empty());
        assert!(lines("classdef (Abstract) C\nend\n", "ATTF").is_empty());
    }
}
