use super::*;

impl GoodPracticesEngine {
    /// LNGNM: Variable name exceeds maximum length.
    pub(crate) fn check_lngnm(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("LNGNM") || node.kind() != "assignment" {
            return Vec::new();
        }

        let lhs = match node.child_by_field_name("left") {
            Some(l) => l,
            None => return Vec::new(),
        };

        if lhs.kind() != "identifier" {
            return Vec::new();
        }

        let name = node_text(lhs, source);
        if name.len() <= self.config.max_variable_name_length {
            return Vec::new();
        }

        let pos = lhs.start_position();
        vec![Diagnostic {
            rule_id: "LNGNM",
            message: format!(
                "Names longer than {} characters are not supported. This name has been truncated to {} characters.",
                self.config.max_variable_name_length, self.config.max_variable_name_length
            ),
            severity: Severity::Warning,
            byte_range: lhs.start_byte()..lhs.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: None,
        }]
    }
}

#[cfg(test)]
mod tests {
    use crate::test_util::{has_id, lint_nodes};
    use malt_core::Config;

    fn lint(source: &str) -> Vec<malt_core::Diagnostic> {
        let engine = crate::good_practices::GoodPracticesEngine::from_config(&Config::default());
        lint_nodes(&*engine, source)
    }

    #[test]
    fn lngnm_silent_on_name_longer_than_63_characters() {
        // Probed in MATLAB R2026b: no LNGNM for names of 63 to 120 characters
        // (script, function, right-hand side, field, loop variable, input, global,
        // cell, names equal in the first 63 characters, digits and capitals).
        // Not emitted until MATLAB reports it (see NOT_REPRODUCED).
        let name = "a".repeat(80);
        for source in [
            format!("{name} = 1;\n"),
            format!("function r = f()\n{name} = 1;\nr = {name};\nend\n"),
            format!("x = 1;\ny = {name} + 1;\n"),
        ] {
            let diags = lint(&source);
            assert!(!has_id(&diags, "LNGNM"), "{source}: got: {diags:?}");
        }
    }
}
