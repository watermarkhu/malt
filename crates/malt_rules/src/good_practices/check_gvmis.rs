use super::*;

impl GoodPracticesEngine {
    /// GVMIS: every `global` declaration statement.
    ///
    /// Probed on the fixtures: MATLAB reports one GVMIS per `global` statement,
    /// at the keyword, in scripts and functions alike (`global G H` once, a
    /// `global` inside an `if` or a loop too). Uses of a global without a
    /// declaration in that scope are not reported.
    pub(crate) fn check_gvmis(&self, tree: &tree_sitter::Tree, _source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("GVMIS") {
            return Vec::new();
        }

        let mut globals = Vec::new();
        collect_nodes_of_kind(tree.root_node(), "global_operator", &mut globals);
        globals
            .into_iter()
            .map(|node| {
                let pos = node.start_position();
                Diagnostic {
                    rule_id: "GVMIS",
                    message: "Global variables are inefficient and make errors difficult to diagnose. Use a function with input variables instead."
                        .to_string(),
                    severity: Severity::Warning,
                    byte_range: node.start_byte()..node.end_byte(),
                    line: pos.row + 1,
                    column: pos.column + 1,
                    fix: None,
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gvmis_lines(source: &str) -> Vec<(usize, usize)> {
        let tree = parse(source);
        engine()
            .check_gvmis(&tree, source)
            .iter()
            .map(|d| (d.line, d.column))
            .collect()
    }

    #[test]
    fn test_gvmis_fires_on_global_declaration_in_function() {
        // Probed (fix/c3__p1.m:2): one diagnostic at the `global` keyword.
        assert_eq!(
            gvmis_lines("function f()\nglobal G\nG = 1;\nend\n"),
            vec![(2, 1)]
        );
    }

    #[test]
    fn test_gvmis_fires_once_for_several_names() {
        // Probed (fix/gp-a__t11.m:1): `global G H` is one statement, one diagnostic.
        assert_eq!(gvmis_lines("global G H\nG = 1;\n"), vec![(1, 1)]);
    }

    #[test]
    fn test_gvmis_fires_on_nested_global_at_keyword_column() {
        // Probed (fix/ls-b__g1.m:7): a `global` inside an `if` is reported at its keyword.
        assert_eq!(
            gvmis_lines("function g()\nif true\n    global q\nend\nend\n"),
            vec![(3, 5)]
        );
    }

    #[test]
    fn test_gvmis_fires_in_script() {
        // Probed (fix/t__scr.m:5).
        assert_eq!(gvmis_lines("x = 1;\nglobal scr\n"), vec![(2, 1)]);
    }

    #[test]
    fn test_gvmis_silent_on_use_without_declaration() {
        // Probed (fix/gp-a__t11.m:9): MATLAB does not report `y = G` in a function
        // that does not declare G.
        assert!(gvmis_lines("function g()\ny = G;\nend\n").is_empty());
    }

    #[test]
    fn test_gvmis_silent_on_persistent() {
        // `persistent` is a different statement (NPERS/PFGP territory), not GVMIS.
        assert!(gvmis_lines("function f()\npersistent p\np = 1;\nend\n").is_empty());
    }
}
