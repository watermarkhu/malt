//! Node-level check for `command` nodes: the `clear` family (CLEAR0ARGS, CLALL, CLCLS, CLFUNC, CLJAVA, CLMEX).

use super::*;

impl PerformanceEngine {
    /// Check a `command` node for clear-related patterns.
    pub(crate) fn check_command<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        source: &'a str,
    ) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        let cmd_name = match extract_command_name(node, source) {
            Some(n) => n,
            None => return diags,
        };

        if cmd_name != "clear" {
            return diags;
        }

        let args = extract_command_args(node, source);

        // CLEAR0ARGS: clear with no arguments
        if self.is_enabled("CLEAR0ARGS") && args.is_empty() {
            diags.push(make_diag("CLEAR0ARGS", node));
        }

        // CLALL: clear all
        if self.is_enabled("CLALL") && args.contains(&"all") {
            diags.push(make_diag("CLALL", node));
        }

        // CLCLS: clear classes
        if self.is_enabled("CLCLS") && args.contains(&"classes") {
            diags.push(make_diag("CLCLS", node));
        }

        // CLFUNC: clear functions
        if self.is_enabled("CLFUNC") && args.contains(&"functions") {
            diags.push(make_diag("CLFUNC", node));
        }

        // CLJAVA: clear java
        if self.is_enabled("CLJAVA") && args.contains(&"java") {
            diags.push(make_diag("CLJAVA", node));
        }

        // CLMEX: clear mex
        if self.is_enabled("CLMEX") && args.contains(&"mex") {
            diags.push(make_diag("CLMEX", node));
        }

        diags
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::engine;
    use crate::test_util::{has_id, lint_nodes};

    // -- clear-family commands -------------------------------------------------

    #[test]
    fn clear0args_fires_on_bare_clear() {
        let src = "clear;\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "CLEAR0ARGS"), "got: {diags:?}");
    }

    #[test]
    fn clall_fires_on_clear_all() {
        let src = "clear all;\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "CLALL"), "got: {diags:?}");
    }

    #[test]
    fn clcls_fires_on_clear_classes() {
        let src = "clear classes;\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "CLCLS"), "got: {diags:?}");
    }

    #[test]
    fn clfunc_fires_on_clear_functions() {
        let src = "clear functions;\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "CLFUNC"), "got: {diags:?}");
    }

    #[test]
    fn cljava_fires_on_clear_java() {
        let src = "clear java;\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "CLJAVA"), "got: {diags:?}");
    }

    #[test]
    fn clmex_fires_on_clear_mex() {
        let src = "clear mex;\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "CLMEX"), "got: {diags:?}");
    }

    #[test]
    fn clear_checks_not_fire_on_named_clear() {
        let src = "clear x;\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "CLEAR0ARGS"), "got: {diags:?}");
        assert!(!has_id(&diags, "CLALL"), "got: {diags:?}");
        assert!(!has_id(&diags, "CLCLS"), "got: {diags:?}");
        assert!(!has_id(&diags, "CLFUNC"), "got: {diags:?}");
        assert!(!has_id(&diags, "CLJAVA"), "got: {diags:?}");
        assert!(!has_id(&diags, "CLMEX"), "got: {diags:?}");
    }
}
