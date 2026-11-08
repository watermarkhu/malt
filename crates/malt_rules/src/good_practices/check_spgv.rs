use super::*;

impl GoodPracticesEngine {
    /// SPGV: use of a GLOBAL or PERSISTENT variable inside an SPMD block.
    pub(crate) fn check_spgv(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("SPGV") || node.kind() != "spmd_statement" {
            return Vec::new();
        }

        let globals = collect_global_persistent_vars(root_node_of(node), source);
        if globals.is_empty() {
            return Vec::new();
        }
        let body = match find_child_of_kind(node, "block") {
            Some(b) => b,
            None => return Vec::new(),
        };

        collect_global_use_diagnostics(body, source, &globals, "SPGV", false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spgv_fires_on_global_use_in_spmd() {
        let source = "global g2;\nspmd\n    z = g2 + 1;\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let spmd = find_child_of_kind(root, "spmd_statement").unwrap();
        let diags = eng.check_spgv(spmd, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "SPGV");
    }

    #[test]
    fn test_spgv_silent_without_globals() {
        let source = "spmd\n    z = 1 + 1;\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let spmd = find_child_of_kind(root, "spmd_statement").unwrap();
        assert!(eng.check_spgv(spmd, source).is_empty());
    }
}
