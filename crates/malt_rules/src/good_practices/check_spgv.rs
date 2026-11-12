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
        let body = find_child_of_kind(node, "block").unwrap_or(node);

        let mut diagnostics = collect_global_use_diagnostics(body, source, &globals, "SPGV", true);
        // A one-line `spmd(2), G, end` parses the bare `G` as a command name, not
        // an identifier. MATLAB reports it as a read (probed).
        let mut commands = Vec::new();
        collect_nodes_of_kind(body, "command", &mut commands);
        for command in commands {
            let Some(name_node) = find_child_of_kind(command, "command_name") else {
                continue;
            };
            let name = node_text(name_node, source);
            if command.named_child_count() != 1 || !globals.contains(name) {
                continue;
            }
            let pos = name_node.start_position();
            diagnostics.push(Diagnostic {
                rule_id: "SPGV",
                message: format!(
                    "Using the GLOBAL or PERSISTENT variable {name} in an SPMD block might fail because it is accessed on a worker machine."
                ),
                severity: Severity::Warning,
                byte_range: name_node.start_byte()..name_node.end_byte(),
                line: pos.row + 1,
                column: pos.column + 1,
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
    fn test_spgv_reports_every_read_and_no_indexed_write() {
        // Probed: `x = G; z = G;` gives SPGV on both lines (sg06); `G(1) = 1` and
        // `G.a = 1` are SPGP, not SPGV (sg05, sg17).
        let count = |body: &str| {
            let src = format!("function f()\nglobal G\nspmd\n    {body}\nend\nend\n");
            let engine = crate::good_practices::GoodPracticesEngine::from_config(
                &malt_core::Config::default(),
            );
            crate::test_util::lint_nodes(&*engine, &src)
                .iter()
                .filter(|d| d.rule_id == "SPGV")
                .count()
        };
        assert_eq!(count("x = G;\n    z = G;"), 2);
        assert_eq!(count("G(1) = 1;"), 0);
        assert_eq!(count("G.a = 1;"), 0);
    }

    #[test]
    fn test_spgv_silent_on_global_write_in_spmd() {
        let source = "global g2;\nspmd\n    g2 = 1;\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let spmd = find_child_of_kind(root, "spmd_statement").unwrap();
        assert!(eng.check_spgv(spmd, source).is_empty());
    }

    #[test]
    fn test_spgv_fires_on_one_line_spmd_command_name() {
        // Probed in MATLAB: the bare `G` after `spmd(2),` is reported at its column.
        let source = "global G\nspmd(2), G, end\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let spmd = find_child_of_kind(root, "spmd_statement").unwrap();
        let diags = eng.check_spgv(spmd, source);
        assert_eq!(diags.len(), 1, "got: {diags:?}");
        assert_eq!((diags[0].line, diags[0].column), (2, 10));
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
