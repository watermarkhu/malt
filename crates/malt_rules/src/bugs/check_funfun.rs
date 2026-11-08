use super::*;

impl BugsEngine {
    /// FUNFUN: Function name passed as string instead of handle to higher-order functions.
    pub(crate) fn check_funfun(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if node.kind() != "function_call" {
            return Vec::new();
        }

        let func_name = match extract_call_name(node, source) {
            Some(n) => n,
            None => return Vec::new(),
        };

        let ho_fns = self.higher_order_functions();
        if !ho_fns.contains(&func_name) {
            return Vec::new();
        }

        // Check if the first argument is a string literal (should be a function handle).
        let args = collect_call_args(node, source);
        if args.is_empty() {
            return Vec::new();
        }

        let first_arg = args[0].trim();
        let is_string = (first_arg.starts_with('\'') && first_arg.ends_with('\''))
            || (first_arg.starts_with('"') && first_arg.ends_with('"'));

        if is_string {
            let fn_name = first_arg.trim_matches('\'').trim_matches('"');
            let pos = node.start_position();
            vec![Diagnostic {
                rule_id: "FUNFUN",
                message: format!(
                    "The first input argument must be a function handle. Did you mean '@{}'?",
                    fn_name
                ),
                severity: Severity::Error,
                byte_range: node.start_byte()..node.end_byte(),
                line: pos.row + 1,
                column: pos.column + 1,
                fix: None,
            }]
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::has_id;

    // -- FUNFUN --------------------------------------------------------------

    #[test]
    fn funfun_fires_on_string_function_arg() {
        let src = "cellfun('isempty', x);\n";
        let diags = node_diags(src);
        assert!(has_id(&diags, "FUNFUN"), "got: {diags:?}");
    }

    #[test]
    fn funfun_no_fire_on_handle_arg() {
        let src = "cellfun(@isempty, x);\n";
        let diags = node_diags(src);
        assert!(!has_id(&diags, "FUNFUN"), "got: {diags:?}");
    }
}
