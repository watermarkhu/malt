//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FVVIN / FVVCON: input validation function rules.
    pub(crate) fn check_fvvin_fvvcon(
        &self,
        blocks: &[ArgumentsBlockMeta],
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let vin = self.is_check_enabled("FVVIN");
        let vcon = self.is_check_enabled("FVVCON");
        if !vin && !vcon {
            return;
        }

        let mut input_props: Vec<(tree_sitter::Node, String)> = Vec::new();
        for block in blocks {
            if !(block.role.is_input() || block.is_input_repeating()) {
                continue;
            }
            for prop in block_properties(block.node) {
                if prop_has_ignored(prop) {
                    continue;
                }
                let name = if prop_has_name_value(prop) {
                    Some(prop_name_value_text(prop, source))
                } else {
                    prop_plain_name(prop, source)
                };
                if let Some(name) = name {
                    input_props.push((prop, name));
                }
            }
        }

        for (index, (prop, name)) in input_props.iter().enumerate() {
            // A varargin is a previously declared name for later validators, not validated itself.
            if name.as_str() == "varargin" {
                continue;
            }
            let calls = prop_validator_calls(*prop, source);
            if calls.is_empty() {
                continue;
            }
            let previously_declared: Vec<&String> =
                input_props[..index].iter().map(|(_, n)| n).collect();
            for call in &calls {
                // A validator without arguments is VTPEAL in MATLAB, not FVVIN (probed).
                if call.no_arguments {
                    continue;
                }
                let uses_arg = call.references.iter().any(|r| {
                    matches!(
                        r,
                        CallReference::Identifier(n) | CallReference::Field(n) if n == name
                    )
                });
                if vin && !uses_arg {
                    self.push_diag(
                        *prop,
                        "FVVIN",
                        "Validation function must use the argument as an input.",
                        diagnostics,
                    );
                }
                if !vcon {
                    continue;
                }
                for reference in &call.references {
                    let allowed = match reference {
                        CallReference::Identifier(ref_name) => {
                            ref_name == name || previously_declared.contains(&ref_name)
                        }
                        CallReference::Field(ref_name) => ref_name == name,
                        CallReference::Other => false,
                    };
                    if !allowed {
                        self.push_diag(
                            *prop,
                            "FVVCON",
                            "For input arguments, validation functions must only use previously declared positional arguments, the argument being validated, or literals.",
                            diagnostics,
                        );
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_fvvcon_no_fire_varargin_is_previously_declared() {
        // Probed: a varargin declared before `x` is a previously declared name (MATLAB: FVREPO only).
        let source = "\
function f(varargin, x)
arguments (Repeating)
    varargin
    x double {mustBeFoo(x, varargin)}
end
end
";
        let diags = check_source(source, "f.m");
        assert!(filter_by_id(&diags, "FVVCON").is_empty(), "got: {diags:?}");
    }

    #[test]
    fn test_fvvcon_fires_forward_reference() {
        let source = "\
function f(a, b)
    arguments
        a (1,1) double {mustBeLessThan(a, b)}
        b (1,1) double
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FVVCON").is_empty(),
            "FVVCON should fire when a validator references a later argument"
        );
        assert!(
            filter_by_id(&diags, "FVUBD").is_empty(),
            "FVUBD is never reported by MATLAB for validators"
        );
    }

    #[test]
    fn test_fvvcon_no_fire_reference_after_declaration() {
        let source = "\
function f(a, b)
    arguments
        b (1,1) double
        a (1,1) double {mustBeLessThan(a, b)}
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVVCON").is_empty(),
            "FVVCON should NOT fire when a validator references an earlier argument"
        );
    }

    #[test]
    fn test_fvvcon_fires_undeclared_reference() {
        let source = "\
function f(a)
    arguments
        a (1,1) double {mustBeLessThan(a, z)}
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FVVCON").is_empty(),
            "FVVCON should fire when a validator references a non-positional value"
        );
    }

    #[test]
    fn test_fvvcon_fires_nested_call_and_builtin_constant() {
        let source = "\
function f(y, z, w)
    arguments
        y
        z (1,1) {mustBeInRange(z, 0, numel(y))}
        w (1,1) {mustBeLessThan(w, pi)}
    end
end
";
        let diags = check_source(source, "f.m");
        assert_eq!(filter_by_id(&diags, "FVVCON").len(), 2);
    }

    #[test]
    fn test_fvvcon_fires_metaclass_operator() {
        let source = "\
function f(x)
    arguments
        x {mustBeA(x, ?matlab.System)}
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FVVCON").is_empty(),
            "FVVCON should fire for a metaclass operator in a validator"
        );
    }

    #[test]
    fn test_fvvcon_no_fire_self_and_prior() {
        let source = "\
function f(a, b)
    arguments
        a (1,1) double {mustBeGreaterThan(a, 0)}
        b (1,1) double
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVVCON").is_empty(),
            "FVVCON should NOT fire when validators use the argument or literals"
        );
    }

    #[test]
    fn test_fvvcon_no_fire_literal_matrix() {
        let source = "\
function f(z)
    arguments
        z (1,1) {mustBeMember(z, [1 2 3])}
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(filter_by_id(&diags, "FVVCON").is_empty());
    }

    #[test]
    fn test_fvvin_and_fvvcon_dotted_name_value() {
        let source = "\
function f(opts)
    arguments
        opts.A (1,1) {mustBeNumeric(A)}
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(!filter_by_id(&diags, "FVVIN").is_empty());
        assert!(!filter_by_id(&diags, "FVVCON").is_empty());
    }

    #[test]
    fn test_fvvin_and_fvvcon_no_fire_dotted_self() {
        let source = "\
function f(opts)
    arguments
        opts.B (1,1) {mustBeNumeric(opts.B)}
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(filter_by_id(&diags, "FVVIN").is_empty());
        assert!(filter_by_id(&diags, "FVVCON").is_empty());
    }

    #[test]
    fn test_fvvin_fires_argument_not_used() {
        let source = "\
function f(a, b)
    arguments
        a (1,1) double {mustBeGreaterThan(b, 0)}
        b (1,1) double
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FVVIN").is_empty(),
            "FVVIN should fire when the validated argument is not used by the validator"
        );
    }

    #[test]
    fn test_fvvin_no_fire_argument_used() {
        let source = "\
function f(a)
    arguments
        a (1,1) double {mustBeReal}
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVVIN").is_empty(),
            "FVVIN should NOT fire when the validator uses the argument (or is bare)"
        );
    }
}
