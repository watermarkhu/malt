//! See the parent module `super` for the shared engine and helpers.

use super::check_parfor_header::literal_value;
use super::*;

impl LanguageSpecEngine {
    /// SPDEC / SPDEC3: validate the worker bounds in an `spmd (n)` or `spmd (m, n)` header.
    ///
    /// Fires SPDEC when a bound is a negative or non-integer value (a literal, or
    /// arithmetic on literals), and SPDEC3 when more than two bounds are given.
    pub(crate) fn check_spmd_worker_bounds(
        &self,
        node: tree_sitter::Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let mut in_parens = false;
        let mut named_bound_count = 0usize;
        let mut bounds: Vec<tree_sitter::Node> = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let text = node_text(child, source);
            if text == "(" {
                in_parens = true;
                continue;
            }
            if text == ")" {
                break;
            }
            if in_parens && child.is_named() {
                named_bound_count += 1;
                bounds.push(child);
            }
        }
        if named_bound_count == 0 {
            return;
        }
        if named_bound_count >= 3 {
            let pos = node.start_position();
            diagnostics.push(Diagnostic {
                rule_id: "SPDEC3",
                message: "An SPMD block can only specify a lower and upper bound for the number of workers to use."
                    .to_string(),
                severity: Severity::Error,
                byte_range: node.start_byte()..node.end_byte(),
                line: pos.row + 1,
                column: pos.column + 1,
                fix: None,
            });
        }
        for bound in bounds {
            // Judged by value (probed): `-0`, `1.0`, `2.0` and `2*0.5` are valid bounds,
            // while `-2`, `2.5` and `1 - 3` are not.
            let is_invalid = literal_value(bound, source, true)
                .is_some_and(|value| value < 0.0 || value.fract() != 0.0);
            if is_invalid {
                let pos = node.start_position();
                diagnostics.push(Diagnostic {
                    rule_id: "SPDEC",
                    message: "The bounds on the number of workers an SPMD block can use must be a nonnegative integer."
                        .to_string(),
                    severity: Severity::Error,
                    byte_range: node.start_byte()..node.end_byte(),
                    line: pos.row + 1,
                    column: pos.column + 1,
                    fix: None,
                });
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_spdec_fires_negative_and_fractional_bounds() {
        let source = "\
function foo()
    spmd (-2, 4)
        x = 1;
    end
    spmd (2.5)
        x = 1;
    end
end
";
        let diags = check_source(source, "foo.m");
        let spdec = filter_by_id(&diags, "SPDEC");
        assert_eq!(
            spdec.len(),
            2,
            "SPDEC should fire for negative/fractional bounds"
        );
    }

    #[test]
    fn test_spdec_no_fire_valid_bounds() {
        let source = "\
function foo()
    spmd (2)
        x = 1;
    end
    spmd (2, 4)
        x = 1;
    end
end
";
        let diags = check_source(source, "foo.m");
        let spdec = filter_by_id(&diags, "SPDEC");
        assert!(
            spdec.is_empty(),
            "SPDEC should NOT fire for valid integer bounds"
        );
    }

    #[test]
    fn test_spdec3_fires_three_bounds() {
        let source = "\
function foo()
    spmd (2, 4, 6)
        x = 1;
    end
end
";
        let diags = check_source(source, "foo.m");
        let spdec3 = filter_by_id(&diags, "SPDEC3");
        assert!(!spdec3.is_empty(), "SPDEC3 should fire for three bounds");
    }

    #[test]
    fn test_spdec3_no_fire_two_bounds() {
        let source = "\
function foo()
    spmd (2, 4)
        x = 1;
    end
end
";
        let diags = check_source(source, "foo.m");
        let spdec3 = filter_by_id(&diags, "SPDEC3");
        assert!(spdec3.is_empty(), "SPDEC3 should NOT fire for two bounds");
    }

    #[test]
    fn test_spdec_fires_arithmetic_negative_bound() {
        // Probed (w18, w23): `spmd (1 - 2)` and `spmd (1 - 3)` are SPDEC.
        let source = "function foo()\n    spmd (1 - 3)\n    end\n    spmd (1 - 2)\n    end\nend\n";
        let diags = check_source(source, "foo.m");
        assert_eq!(filter_by_id(&diags, "SPDEC").len(), 2, "got: {diags:?}");
    }

    #[test]
    fn test_spdec_no_fire_integer_valued_bounds() {
        // Probed (w07, w13, w15, w21, w22): `1.0`, `-0`, `2.0` and `2*0.5`, `4/2` are valid.
        let source = "\
function foo()
    spmd (1.0)
    end
    spmd (-0)
    end
    spmd (2.0, 3)
    end
    spmd (2*0.5)
    end
    spmd (4/2)
    end
end
";
        let diags = check_source(source, "foo.m");
        assert!(filter_by_id(&diags, "SPDEC").is_empty(), "got: {diags:?}");
    }
}
