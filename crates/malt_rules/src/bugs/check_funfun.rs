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
        // `cellfun` accepts a few legacy function names as strings.
        const CELLFUN_LEGACY: &[&str] = &[
            "isempty",
            "islogical",
            "isreal",
            "length",
            "ndims",
            "prodofsize",
            "size",
            "isclass",
        ];
        let first_name = args[0].trim().trim_matches(|c| c == '\'' || c == '"');
        if func_name == "cellfun" && CELLFUN_LEGACY.contains(&first_name) {
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
                // `cellfun('myfun', c)` becomes `cellfun(@myfun, c)`. Unsafe: a name
                // that is a variable, not a function, would stop working.
                fix: first_arg_node(node)
                    .filter(|arg| {
                        crate::fix_util::simple_literal_text(*arg, source)
                            .is_some_and(crate::fix_util::is_plain_identifier)
                    })
                    .map(|arg| {
                        malt_core::Fix::new(arg.byte_range(), format!("@{fn_name}")).unsafe_fix()
                    }),
            }]
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{fix_of, has_id, reported_without_fix};

    #[test]
    fn funfun_fix_turns_the_name_into_a_handle() {
        let eng = engine();
        let (safe, fixed) = fix_of(&*eng, "y = arrayfun('myfun', x);\n", "FUNFUN").unwrap();
        assert!(!safe);
        assert_eq!(fixed, "y = arrayfun(@myfun, x);\n");
        let (_, fixed) = fix_of(&*eng, "structfun(\"disp\", s);\n", "FUNFUN").unwrap();
        assert_eq!(fixed, "structfun(@disp, s);\n");
        assert!(!has_id(&node_diags(&fixed), "FUNFUN"));
    }

    #[test]
    fn funfun_no_fix_for_a_text_that_is_not_a_name() {
        assert!(reported_without_fix(
            &*engine(),
            "arrayfun('x + 1', v);\n",
            "FUNFUN"
        ));
        assert!(reported_without_fix(
            &*engine(),
            "arrayfun('a.b', v);\n",
            "FUNFUN"
        ));
    }

    // -- FUNFUN --------------------------------------------------------------

    #[test]
    fn funfun_fires_on_string_function_arg() {
        let src = "arrayfun('myfun', x);\n";
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

#[cfg(test)]
mod funfun_legacy_tests {
    use super::*;
    use crate::test_util::has_id;

    #[test]
    fn funfun_no_fire_on_cellfun_legacy_name() {
        let diags = node_diags("cellfun('isempty', x);\n");
        assert!(!has_id(&diags, "FUNFUN"), "got: {diags:?}");
    }
}
