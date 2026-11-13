//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// Check the parfor header: loop variable name (PFANSLP) and range (PFRNG).
    pub(crate) fn check_parfor_header(
        &self,
        node: tree_sitter::Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        // PFANSLP: 'ans' as the parfor loop variable
        if let Some(var) = self.extract_for_variable(node, source) {
            if var == "ans" {
                self.push_diag(
                    node,
                    "PFANSLP",
                    "'ans' is not supported as a parfor loop variable.",
                    diagnostics,
                );
            }
        }

        // PFRNG: a literal start or stop that is not an integer (`1.5:4`, `1:4.5`).
        // Non-unit steps are PFRNI, and bounds that are variables are not evaluated.
        // The diagnostic sits on the colon before the stop value (probed).
        if let Some(range_node) = self.parfor_range_node(node) {
            let mut cursor = range_node.walk();
            let named: Vec<tree_sitter::Node> = range_node.named_children(&mut cursor).collect();
            let is_fractional = |n: &tree_sitter::Node| {
                literal_value(*n, source, false).is_some_and(|v| v.fract() != 0.0)
            };
            if named.len() >= 2
                && (is_fractional(&named[0]) || is_fractional(&named[named.len() - 1]))
            {
                let mut cursor = range_node.walk();
                let stop_colon = range_node
                    .children(&mut cursor)
                    .filter(|c| c.kind() == ":")
                    .last()
                    .unwrap_or(range_node);
                self.push_diag(
                    stop_colon,
                    "PFRNG",
                    "The range of a PARFOR statement must be consecutive integers.",
                    diagnostics,
                );
            }
        }
    }

    /// Find the `range` node of a for/parfor statement's iterator, if any.
    pub(crate) fn parfor_range_node<'a>(
        &self,
        node: tree_sitter::Node<'a>,
    ) -> Option<tree_sitter::Node<'a>> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "iterator" {
                let mut inner = child.walk();
                for c in child.children(&mut inner) {
                    if c.kind() == "range" {
                        return Some(c);
                    }
                }
            }
        }
        None
    }
}

/// The value of a literal: a number, optionally signed or in parentheses (`1.5`,
/// `-1.5`, `(4.5)`). With `binary`, `+ - * /` of such values too. Probed: a parfor
/// bound `1:(4.5)` and `1:-4.5` is PFRNG, but `1:(2+0.5)` is silent; a worker bound
/// `spmd (1 - 3)` is SPDEC.
pub(crate) fn literal_value(node: tree_sitter::Node, source: &str, binary: bool) -> Option<f64> {
    match node.kind() {
        "number" => node_text(node, source).trim().parse::<f64>().ok(),
        "unary_operator" => {
            let value = literal_value(single_named_child(node)?, source, binary)?;
            match node_text(node, source).trim_start().chars().next()? {
                '-' => Some(-value),
                '+' => Some(value),
                _ => None,
            }
        }
        "parenthesis" => literal_value(single_named_child(node)?, source, binary),
        "binary_operator" if binary => {
            let left = literal_value(node.child_by_field_name("left")?, source, binary)?;
            let right = literal_value(node.child_by_field_name("right")?, source, binary)?;
            let mut cursor = node.walk();
            let operator = node
                .children(&mut cursor)
                .find(|c| !c.is_named())
                .map(|c| node_text(c, source))?;
            match operator {
                "+" => Some(left + right),
                "-" => Some(left - right),
                "*" => Some(left * right),
                "/" => Some(left / right),
                _ => None,
            }
        }
        _ => None,
    }
}

/// The only named child of a node, if it has exactly one.
fn single_named_child(node: tree_sitter::Node) -> Option<tree_sitter::Node> {
    let mut cursor = node.walk();
    let mut named = node.named_children(&mut cursor);
    let child = named.next()?;
    named.next().is_none().then_some(child)
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_pfanslp_fires_ans_parfor_variable() {
        let source = "\
function f()
    parfor ans = 1:10
        y = ans;
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "PFANSLP").is_empty(),
            "PFANSLP should fire for 'ans' as parfor variable"
        );
    }

    #[test]
    fn test_pfanslp_no_fire_normal_parfor_variable() {
        let source = "\
function f()
    parfor i = 1:10
        y = i;
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "PFANSLP").is_empty(),
            "PFANSLP should NOT fire for a normal parfor variable"
        );
    }

    #[test]
    fn test_pfrng_no_fire_integer_step() {
        // Probed: `1:2:10` is PFRNI only, not PFRNG.
        let source = "\
function f()
    parfor i = 1:2:10
        x(i) = i;
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "PFRNG").is_empty(),
            "PFRNG should NOT fire for an integer step (that is PFRNI)"
        );
    }

    #[test]
    fn test_pfrng_fires_fractional_start_and_stop() {
        // Probed at column 1: `1.5:4` reports at the colon before `4` (column 15),
        // `1:4.5` at the colon before `4.5` (column 13). Indented by four here.
        let source = "\
function f()
    parfor i = 1.5:4
        x(i) = i;
    end
    parfor i = 1:4.5
        x(i) = i;
    end
end
";
        let diags = check_source(source, "f.m");
        let pfrng = filter_by_id(&diags, "PFRNG");
        assert_eq!(pfrng.len(), 2, "got: {diags:?}");
        assert_eq!((pfrng[0].line, pfrng[0].column), (2, 19));
        assert_eq!((pfrng[1].line, pfrng[1].column), (5, 17));
    }

    #[test]
    fn test_pfrng_no_fire_variable_bounds() {
        // Probed: MATLAB does not report bounds it cannot evaluate (`1:n` with a variable).
        let source = "\
function f()
    n = 2;
    parfor i = 1:n
        x(i) = i;
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(filter_by_id(&diags, "PFRNG").is_empty());
    }

    #[test]
    fn test_pfrng_no_fire_negative_step() {
        // Probed: `10:-1:1` is not reported by MATLAB.
        let source = "\
function f()
    parfor i = 10:-1:1
        x(i) = i;
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "PFRNG").is_empty(),
            "PFRNG should NOT fire for a negative integer step"
        );
    }

    #[test]
    fn test_pfrng_no_fire_unit_step() {
        let source = "\
function f()
    parfor i = 1:10
        x(i) = i;
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "PFRNG").is_empty(),
            "PFRNG should NOT fire for a unit step"
        );
    }

    #[test]
    fn test_pfrng_fires_signed_and_parenthesised_fractional_bounds() {
        // Probed (h06, h12, h22, h24): `-1.5:4`, `(1.5):4`, `1:(4.5)` and `1:-4.5` are PFRNG.
        let source = "\
function f()
    parfor i = -1.5:4
    end
    parfor i = (1.5):4
    end
    parfor i = 1:(4.5)
    end
    parfor i = 1:-4.5
    end
end
";
        let diags = check_source(source, "f.m");
        assert_eq!(filter_by_id(&diags, "PFRNG").len(), 4, "got: {diags:?}");
    }

    #[test]
    fn test_pfrng_silent_on_binary_bound() {
        // Probed (h26): MATLAB does not evaluate `1:(2+0.5)` in a parfor header.
        let source = "function f()\n    parfor i = 1:(2+0.5)\n    end\nend\n";
        let diags = check_source(source, "f.m");
        assert!(filter_by_id(&diags, "PFRNG").is_empty(), "got: {diags:?}");
    }
}
