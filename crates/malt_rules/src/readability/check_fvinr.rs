//! # FVINR: add an `(Input)` attribute to `arguments` blocks
//!
//! `check_fvinr` flags `arguments` blocks that declare no `(Input)` or
//! `(Output)` attribute and inserts `(Input)` for readability.

use super::*;

/// Whether an `arguments` block carries an `(Output)` attribute.
fn has_output_attribute(block: tree_sitter::Node, source: &str) -> bool {
    block.named_children(&mut block.walk()).any(|child| {
        child.kind() == "attributes"
            && source[child.start_byte()..child.end_byte()].contains("Output")
    })
}

/// Whether another `arguments` block with an `(Output)` attribute sits in the
/// same function. The search covers the whole subtree, since a file with a
/// syntax error can nest the blocks under an error node.
fn output_block_in_scope(node: tree_sitter::Node, source: &str) -> bool {
    let mut scope = node;
    while let Some(parent) = scope.parent() {
        scope = parent;
        if scope.kind() == "function_definition" {
            break;
        }
    }
    let mut stack = vec![scope];
    while let Some(current) = stack.pop() {
        if current.id() != node.id()
            && current.kind() == "arguments_statement"
            && has_output_attribute(current, source)
        {
            return true;
        }
        let mut cursor = current.walk();
        stack.extend(current.named_children(&mut cursor));
    }
    false
}

impl ReadabilityEngine {
    /// FVINR: Add `(Input)` attribute to `arguments` block for readability.
    pub(crate) fn check_fvinr(&self, ctx: &NodeContext, results: &mut Vec<Diagnostic>) {
        if !self.is_check_enabled("FVINR") {
            return;
        }
        let node = ctx.node;

        // If any `attributes` child exists, the block already has (Input) or (Output).
        let has_attributes = node
            .named_children(&mut node.walk())
            .any(|c| c.kind() == "attributes");
        if has_attributes {
            return;
        }

        // MATLAB reports an unattributed block only when the function also has an
        // `arguments (Output)` block. A lone block, or two unattributed blocks,
        // is silent (probes fv1, fv6 and q6).
        let has_output_block = output_block_in_scope(node, ctx.source);
        if !has_output_block {
            return;
        }

        // The `arguments` keyword is the first (unnamed) child.
        let arguments_keyword = match node.child(0) {
            Some(c) if c.kind() == "arguments" => c,
            _ => return,
        };

        // CRITICAL: fix text is " (Input)" — LEADING space, no trailing space.
        // The keyword is immediately followed by a newline, so inserting a
        // trailing space would produce `arguments(Input)` (invalid).
        let fix = Fix::insert(arguments_keyword.end_byte(), " (Input)");
        results.push(self.diag(
            "FVINR",
            "For readability, add Input attribute to the input arguments block.",
            arguments_keyword,
            Some(fix),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_nodes};
    use malt_core::Config;

    fn engine() -> Box<dyn Rule> {
        ReadabilityEngine::from_config(&Config::default())
    }

    // -- FVINR ---------------------------------------------------------------

    #[test]
    fn fvinr_no_attribute_fires() {
        let source = "function f(a)\n    arguments\n        a (1,1)\n    end\n    arguments (Output)\n    end\nend\n";
        let diags = lint_nodes(&*engine(), source);
        assert!(has_id(&diags, "FVINR"), "got: {diags:?}");
    }

    #[test]
    fn fvinr_lone_block_does_not_fire() {
        // MATLAB is silent on a lone unattributed block (probe fv1).
        let source = "function f(a)\n    arguments\n        a (1,1)\n    end\nend\n";
        let diags = lint_nodes(&*engine(), source);
        assert!(!has_id(&diags, "FVINR"), "got: {diags:?}");
    }

    #[test]
    fn fvinr_with_input_attribute_does_not_fire() {
        let source = "function f(a)\n    arguments (Input)\n        a (1,1)\n    end\nend\n";
        let diags = lint_nodes(&*engine(), source);
        assert!(!has_id(&diags, "FVINR"), "got: {diags:?}");
    }

    #[test]
    fn fvinr_with_output_attribute_does_not_fire() {
        let source = "function f(a)\n    arguments (Output)\n        a\n    end\nend\n";
        let diags = lint_nodes(&*engine(), source);
        assert!(!has_id(&diags, "FVINR"), "got: {diags:?}");
    }

    #[test]
    fn fvinr_disabled_in_config_does_not_fire() {
        let config =
            Config::from_toml("[lint.rules.READABILITY_ENGINE]\ndisabled_checks = [\"FVINR\"]\n")
                .expect("valid config");
        let rule = ReadabilityEngine::from_config(&config);
        let source = "function f(a)\n    arguments\n        a (1,1)\n    end\n    arguments (Output)\n    end\nend\n";
        let diags = lint_nodes(&*rule, source);
        assert!(!has_id(&diags, "FVINR"), "got: {diags:?}");
    }

    #[test]
    fn fvinr_fix_inserts_input_attribute() {
        let source = "function f(a)\n    arguments\n        a (1,1)\n    end\n    arguments (Output)\n    end\nend\n";
        let diags = lint_nodes(&*engine(), source);
        let diag = diags
            .iter()
            .find(|d| d.rule_id == "FVINR")
            .expect("FVINR should fire");
        let fix = diag.fix.as_ref().expect("FVINR should carry an auto-fix");
        assert_eq!(fix.replacement, " (Input)");
        let mut fixed = String::from(source);
        fixed.replace_range(fix.byte_range.clone(), &fix.replacement);
        assert!(fixed.contains("arguments (Input)"), "got: {fixed:?}");
    }
}
