use super::*;

impl GoodPracticesEngine {
    /// DUALC: a command with arguments is followed by a comma and then a bare
    /// identifier, as in `clear x, y` or `disp hi, there`, where the comma can
    /// end the command early. Probed: MATLAB is silent when the comma is followed
    /// by a call, an assignment or another command, and when the command has no
    /// arguments (`hold on, plot(x)`, `hold on, x = 3`, `disp hi, disp there`,
    /// `format long, format short`, `clear, clc`).
    pub(crate) fn check_dualc(&self, node: Node, _source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("DUALC") || node.kind() != "command" {
            return Vec::new();
        }
        // The command name and at least one argument.
        if node.named_child_count() < 2 {
            return Vec::new();
        }

        // A bare word after the comma parses as a command with only its name.
        let bare_identifier_after_comma = node.next_sibling().is_some_and(|comma| {
            comma.kind() == ","
                && comma
                    .next_named_sibling()
                    .is_some_and(|n| n.kind() == "command" && n.named_child_count() == 1)
        });
        if !bare_identifier_after_comma {
            return Vec::new();
        }

        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: "DUALC",
            message: "Command might be prematurely ended by comma.".to_string(),
            severity: Severity::Warning,
            byte_range: node.start_byte()..node.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: None,
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_nodes};

    /// DUALC diagnostics for every command in `source`.
    fn dualc(source: &str) -> Vec<Diagnostic> {
        let tree = parse(source);
        let eng = engine();
        let mut out = Vec::new();
        let mut stack = vec![tree.root_node()];
        while let Some(n) = stack.pop() {
            out.extend(eng.check_dualc(n, source));
            let mut c = n.walk();
            stack.extend(n.children(&mut c));
        }
        out
    }

    #[test]
    fn test_dualc_fires_on_bare_identifier_after_comma() {
        // Probed: MATLAB reports each of these once, at the command's start.
        for source in [
            "clear x, y\n",
            "disp hi, there\n",
            "disp hello, world\n",
            "hold on , clc;\n",
            "clear x, y, z\n",
            "format long, x\n",
        ] {
            let diags = dualc(source);
            assert_eq!(diags.len(), 1, "{source}");
            assert_eq!(diags[0].rule_id, "DUALC");
            assert_eq!((diags[0].line, diags[0].column), (1, 1));
        }
    }

    #[test]
    fn test_dualc_silent_when_comma_is_followed_by_a_call_assignment_or_command() {
        for source in [
            "hold on, plot(x)\n",
            "hold on, x = 3\n",
            "disp hi, disp there\n",
            "format long, format short\n",
            "clear, clc\n",
            "disp hi\n",
            "more off,\n",
        ] {
            assert!(dualc(source).is_empty(), "{source}");
        }
    }

    #[test]
    fn test_dualc_on_under_mathworks() {
        // MATLAB reports DUALC with its default analysis, so the mathworks preset
        // enables it.
        let mathworks = GoodPracticesEngine::from_config(
            &Config::from_toml("[lint]\npreset = \"mathworks\"\n").unwrap(),
        );
        assert!(has_id(&lint_nodes(&*mathworks, "clear x, y\n"), "DUALC"));
    }
}
