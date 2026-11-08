//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// ERTXT / WTXT: `error`/`warning` called with a single string argument that
    /// is a message identifier (contains `:`) and no message text.
    pub(crate) fn check_error_warning_message(
        &self,
        node: tree_sitter::Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let Some(name) = callee_name(node, source) else {
            return;
        };
        let (rule_id, message) = match name.as_str() {
            "error" => (
                "ERTXT",
                "Specify an error message with the message identifier.",
            ),
            "warning" => (
                "WTXT",
                "Specify a warning message with the message identifier.",
            ),
            _ => return,
        };
        let args = argument_nodes(node);
        if args.len() != 1 {
            return;
        }
        let Some(content) = string_content(args[0], source) else {
            return;
        };
        if content.contains(':') {
            self.push_diag(node, rule_id, message, diagnostics);
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_ertxt_fires_id_with_no_message() {
        let source = "error('MyTool:badInput');\n";
        let diags = check_source(source, "myscript.m");
        let ertxt = filter_by_id(&diags, "ERTXT");
        assert!(
            !ertxt.is_empty(),
            "ERTXT should fire for error ID with no message"
        );
    }

    #[test]
    fn test_ertxt_no_fire_plain_message_or_id_with_message() {
        let source = "\
error('plain message');
error('MyID', 'message here');
";
        let diags = check_source(source, "myscript.m");
        let ertxt = filter_by_id(&diags, "ERTXT");
        assert!(
            ertxt.is_empty(),
            "ERTXT should NOT fire for plain message or ID+message"
        );
    }

    #[test]
    fn test_wtxt_fires_id_with_no_message() {
        let source = "warning('MyTool:badInput');\n";
        let diags = check_source(source, "myscript.m");
        let wtxt = filter_by_id(&diags, "WTXT");
        assert!(
            !wtxt.is_empty(),
            "WTXT should fire for warning ID with no message"
        );
    }

    #[test]
    fn test_wtxt_no_fire_plain_message() {
        let source = "warning('plain message');\nwarning('MyID', 'msg');\n";
        let diags = check_source(source, "myscript.m");
        let wtxt = filter_by_id(&diags, "WTXT");
        assert!(
            wtxt.is_empty(),
            "WTXT should NOT fire for plain message or ID+message"
        );
    }
}
