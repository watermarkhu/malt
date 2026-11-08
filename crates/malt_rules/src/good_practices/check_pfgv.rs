use super::*;

impl GoodPracticesEngine {
    /// PFGV: use of a GLOBAL variable inside a PARFOR loop.
    pub(crate) fn check_pfgv(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("PFGV") || !is_parfor_node(node, source) {
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

        collect_global_use_diagnostics(body, source, &globals, "PFGV", true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pfgv_fires_on_global_use() {
        let source = "global gVar;\nparfor i = 1:10\n    gVar = i;\n    x(i) = gVar;\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let parfor = find_child_of_kind(root, "for_statement").unwrap();
        let diags = eng.check_pfgv(parfor, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "PFGV");
    }

    #[test]
    fn test_pfgv_silent_without_globals() {
        let source = "parfor i = 1:10\n    q = i;\n    x(i) = q;\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let parfor = find_child_of_kind(root, "for_statement").unwrap();
        assert!(eng.check_pfgv(parfor, source).is_empty());
    }
}
