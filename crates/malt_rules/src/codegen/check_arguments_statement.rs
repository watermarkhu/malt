use super::*;

impl CodegenEngine {
    /// Check an `arguments_statement` node (EMRIFAV): a `Repeating`
    /// arguments block that declares validation.
    pub(crate) fn check_arguments_statement<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        source: &'a str,
    ) -> Vec<Diagnostic> {
        if !self.is_enabled("EMRIFAV") {
            return Vec::new();
        }
        let text = node_text(node, source);
        let mut lines = text.lines();
        let header = lines.next().unwrap_or("");
        if !header.contains("Repeating") {
            return Vec::new();
        }
        // Any declaration with more than a bare name carries validation
        // (size, class, validator functions) or a default.
        let has_validation = lines
            .map(|l| l.split('%').next().unwrap_or("").trim())
            .filter(|l| !l.is_empty() && *l != "end")
            .any(|l| l.split_whitespace().count() > 1);
        if has_validation {
            vec![make_diag("EMRIFAV", node)]
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_nodes};

    // -- EMRIFAV ---------------------------------------------------------------

    #[test]
    fn emrifav_fires_on_repeating_block_with_validation() {
        let src = "function f(varargin)\n    arguments (Repeating)\n        varargin (1,1) double\n    end\nend\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "EMRIFAV"), "got: {diags:?}");
    }

    #[test]
    fn emrifav_not_fire_on_plain_arguments_block() {
        let src =
            "function f(a)\n    arguments\n        a (1,1) double\n    end\n    x = a;\nend\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "EMRIFAV"), "got: {diags:?}");
    }

    #[test]
    fn emrifav_not_fire_without_arguments_block() {
        let src = "function f(a)\n    x = a;\nend\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "EMRIFAV"), "got: {diags:?}");
    }
}
