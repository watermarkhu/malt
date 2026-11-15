//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// USESWNS: in scripts, a variable must be explicitly defined before first use.
    ///
    /// Overlaps with the Warning-severity `SUSENS` from the unset-variables engine;
    /// this is the Error-severity language-specification variant. Users can disable
    /// either engine via configuration.
    pub(crate) fn check_useswns(
        &self,
        symbol_table: &SymbolTable,
        ctx: &FileContext,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("USESWNS") {
            return;
        }
        let root_scope = symbol_table.root_scope();
        let skipped = Self::collect_function_name_starts(ctx.tree.root_node());
        let mut reported: HashSet<String> = HashSet::new();
        // Where the earliest definition of each name starts.
        let mut earliest: HashMap<&str, usize> = HashMap::new();
        for def in &root_scope.defs {
            let start = earliest.entry(def.name.as_str()).or_insert(usize::MAX);
            *start = (*start).min(def.byte_range.start);
        }
        for use_ in &root_scope.uses {
            if skipped.contains(&use_.byte_range.start) {
                continue;
            }
            if reported.contains(&use_.name) {
                continue;
            }
            if let Some(&def_start) = earliest.get(use_.name.as_str()) {
                if use_.byte_range.start < def_start {
                    reported.insert(use_.name.clone());
                    diagnostics.push(Diagnostic {
                        rule_id: "USESWNS",
                        message: "Variable must be explicitly defined before first use."
                            .to_string(),
                        severity: Severity::Error,
                        byte_range: use_.byte_range.clone(),
                        line: use_.line,
                        column: use_.column,
                        fix: None,
                    });
                }
            }
        }
    }

    /// Collect the byte offsets of identifiers that name functions (function-call
    /// callees and command names) so they are not treated as variable uses.
    pub(crate) fn collect_function_name_starts(root: tree_sitter::Node) -> HashSet<usize> {
        let mut starts = HashSet::new();
        Self::collect_function_name_starts_dfs(root, &mut starts);
        starts
    }

    /// DFS helper for [`Self::collect_function_name_starts`].
    pub(crate) fn collect_function_name_starts_dfs(
        node: tree_sitter::Node,
        starts: &mut HashSet<usize>,
    ) {
        match node.kind() {
            "function_call" => {
                if let Some(callee) = crate::analysis::callee::call_name_node(node) {
                    starts.insert(callee.start_byte());
                }
            }
            "command" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "command_name" {
                        starts.insert(child.start_byte());
                    }
                }
            }
            _ => {}
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::collect_function_name_starts_dfs(child, starts);
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id, parse_matlab};
    use crate::language_spec::LanguageSpecEngine;
    use malt_core::{Config, FileContext};
    use std::path::Path;

    /// Run the language spec engine with the `all` preset, where USESWNS is on.
    fn check_source_all(source: &str) -> Vec<malt_core::Diagnostic> {
        let config = Config::from_toml("[lint]\npreset = \"all\"\n").unwrap();
        let rule = LanguageSpecEngine::from_config(&config);
        let tree = parse_matlab(source);
        let ctx = FileContext {
            tree: &tree,
            source,
            file_path: Path::new("myscript.m"),
            project: None,
        };
        rule.check_file(&ctx)
    }

    #[test]
    fn test_useswns_fires_use_before_definition() {
        // MATLAB's default checkcode never emits USESWNS (probed), so it is off
        // under the mathworks preset and only fires under `all`.
        let source = "y = x + 1;\nx = 5;\n";
        let diags = check_source_all(source);
        let useswns = filter_by_id(&diags, "USESWNS");
        assert!(
            !useswns.is_empty(),
            "USESWNS should fire for variable used before definition"
        );
    }

    #[test]
    fn test_useswns_off_under_mathworks() {
        let diags = check_source("y = x + 1;\nx = 5;\n", "myscript.m");
        assert!(filter_by_id(&diags, "USESWNS").is_empty());
    }

    #[test]
    fn test_useswns_no_fire_defined_before_use() {
        let source = "x = 5;\ny = x + 1;\n";
        let diags = check_source(source, "myscript.m");
        let useswns = filter_by_id(&diags, "USESWNS");
        assert!(
            useswns.is_empty(),
            "USESWNS should NOT fire when variable is defined before use"
        );
    }

    #[test]
    fn test_useswns_no_fire_function_calls() {
        let source = "disp('hello');\nplot(1:10);\n";
        let diags = check_source(source, "myscript.m");
        let useswns = filter_by_id(&diags, "USESWNS");
        assert!(
            useswns.is_empty(),
            "USESWNS should NOT fire for function names in a script"
        );
    }
}
