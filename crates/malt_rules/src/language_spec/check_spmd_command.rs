//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// SPLD / SPSV / SPWHOS: command-form transparency checks inside an spmd block.
    pub(crate) fn check_spmd_command(
        &self,
        node: tree_sitter::Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let mut command_name: Option<String> = None;
        let mut arguments: Vec<String> = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "command_name" => {
                    command_name = Some(node_text(child, source).to_string());
                }
                "command_argument" => {
                    arguments.push(node_text(child, source).to_string());
                }
                _ => {}
            }
        }
        let Some(name) = command_name else {
            return;
        };
        match name.as_str() {
            "load" => {
                self.push_diag(node, "SPLD", "To avoid a transparency violation, assign the output of LOAD to a variable in SPMD blocks.", diagnostics);
            }
            "save" if !arguments.iter().any(|a| a == "-fromstruct") => {
                self.push_diag(
                    node,
                    "SPSV",
                    "SAVE cannot be called in an SPMD block without the '-fromstruct' option.",
                    diagnostics,
                );
            }
            "who" | "whos" if !arguments.iter().any(|a| a == "-file") => {
                self.push_diag(node, "SPWHOS", "Using \"who\" or \"whos\" without \"-file\" is invalid inside an SPMD block because it accesses the workspace in a non-transparent way.", diagnostics);
            }
            _ => {}
        }
    }
}
