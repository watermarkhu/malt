//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// TINVALDIM / TTOOFEWDIMS: size constraints must have at least two
    /// nonnegative integer (or colon) dimensions.
    pub(crate) fn check_fvtinvaldim_ttoofewdims(
        &self,
        blocks: &[ArgumentsBlockMeta],
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let invalid = self.is_check_enabled("TINVALDIM");
        let too_few = self.is_check_enabled("TTOOFEWDIMS");
        if !invalid && !too_few {
            return;
        }
        for block in blocks {
            for prop in block_properties(block.node) {
                let Some(dims) = find_child_kind(prop, "dimensions") else {
                    continue;
                };
                let mut entries = Vec::new();
                let mut cursor = dims.walk();
                for child in dims.children(&mut cursor) {
                    match child.kind() {
                        "(" | ")" | "," => {}
                        // A `...` continuation or a comment is not a dimension.
                        "line_continuation" | "comment" => {}
                        _ => entries.push(child),
                    }
                }
                if too_few && entries.len() < 2 {
                    self.push_diag(
                        dims,
                        "TTOOFEWDIMS",
                        "Specify at least two dimensions for size.",
                        diagnostics,
                    );
                }
                if invalid {
                    for entry in &entries {
                        let valid = match entry.kind() {
                            "spread_operator" => true,
                            "number" => {
                                let text = node_text(*entry, source);
                                let value = if let Some(hex) =
                                    text.strip_prefix("0x").or(text.strip_prefix("0X"))
                                {
                                    u64::from_str_radix(hex, 16).ok().map(|v| v as f64)
                                } else if let Some(bin) =
                                    text.strip_prefix("0b").or(text.strip_prefix("0B"))
                                {
                                    u64::from_str_radix(bin, 2).ok().map(|v| v as f64)
                                } else {
                                    text.parse::<f64>().ok()
                                };
                                value.is_some_and(|v| v >= 0.0 && v.fract() == 0.0)
                            }
                            _ => false,
                        };
                        if !valid {
                            self.push_diag(
                                dims,
                                "TINVALDIM",
                                "Each dimension must be a nonnegative integer number or a colon.",
                                diagnostics,
                            );
                            break;
                        }
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
    fn test_tinvaldim_no_fire_continuation_inside_dimensions() {
        // Probed: a `...` continuation inside the size is not a dimension (MATLAB: silent).
        let source = "\
function f(a)
arguments
    a (1, ...
        2) double
end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "TINVALDIM").is_empty(),
            "got: {diags:?}"
        );
    }

    #[test]
    fn test_tinvaldim_fires_identifier_and_negative_dimensions() {
        // Probed: an identifier (`n`) and a negative number (`-2`) are invalid dimensions.
        let source = "\
function f(a, n)
arguments
    a (1, n) double
    n (1, -2) double
end
end
";
        let diags = check_source(source, "f.m");
        assert_eq!(filter_by_id(&diags, "TINVALDIM").len(), 2, "got: {diags:?}");
    }
    #[test]
    fn test_tinvaldim_fires_float_dimension() {
        let source = "\
function f(a)
    arguments
        a (1.5, 2) double
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "TINVALDIM").is_empty(),
            "TINVALDIM should fire for a non-integer dimension"
        );
    }

    #[test]
    fn test_tinvaldim_no_fire_valid_dimensions() {
        let source = "\
function f(a)
    arguments
        a (1, :) double
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "TINVALDIM").is_empty(),
            "TINVALDIM should NOT fire for integer or colon dimensions"
        );
    }

    #[test]
    fn test_tinvaldim_no_fire_hex_and_binary_dimensions() {
        let source = "\
function f(a)
    arguments
        a (0x10, 0b11) double
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "TINVALDIM").is_empty(),
            "TINVALDIM should NOT fire for hex or binary integer dimensions"
        );
    }

    #[test]
    fn test_ttoofewdims_fires_single_dimension() {
        let source = "\
function f(a)
    arguments
        a (1) double
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "TTOOFEWDIMS").is_empty(),
            "TTOOFEWDIMS should fire for a single dimension"
        );
    }

    #[test]
    fn test_ttoofewdims_no_fire_two_dimensions() {
        let source = "\
function f(a)
    arguments
        a (1, 2) double
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "TTOOFEWDIMS").is_empty(),
            "TTOOFEWDIMS should NOT fire for two dimensions"
        );
    }
}
