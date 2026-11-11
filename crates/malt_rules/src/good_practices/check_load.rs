use super::*;

impl GoodPracticesEngine {
    /// LOAD: `load` called without output variable.
    pub(crate) fn check_load(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("LOAD") {
            return Vec::new();
        }

        // Only fire on statement-level load calls (not `x = load(...)`), or on any
        // load call inside an anonymous function (probed: `@() load('f.mat')` in
        // an argument, an assignment or a cellfun reports LOAD). MATLAB reports
        // LOAD only inside a function body (probed; scripts are silent).
        let is_statement = is_statement_level(node);
        if !(is_statement || in_anonymous_function(node)) || !in_function(node) {
            return Vec::new();
        }

        let func_name = match node.kind() {
            "function_call" => get_function_call_name(node, source),
            "command" => get_command_name(node, source),
            _ => return Vec::new(),
        };

        if func_name != Some("load") {
            return Vec::new();
        }
        // `load('f.mat', 'x')` names variables; options such as `-mat` do not count.
        if file_argument_count(node, source) > 1 {
            return Vec::new();
        }

        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: "LOAD",
            message: "To avoid conflicts with functions on the path, specify variables to load from file."
                .to_string(),
            severity: Severity::Warning,
            byte_range: node.start_byte()..node.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: None,
        }]
    }
}

/// Whether `node` sits inside an anonymous function body, within the enclosing function.
fn in_anonymous_function(node: Node) -> bool {
    let mut current = node.parent();
    while let Some(parent) = current {
        match parent.kind() {
            "lambda" => return true,
            "function_definition" => return false,
            _ => current = parent.parent(),
        }
    }
    false
}

/// Whether `node` sits inside a function body.
fn in_function(node: Node) -> bool {
    let mut current = node.parent();
    while let Some(parent) = current {
        if parent.kind() == "function_definition" {
            return true;
        }
        current = parent.parent();
    }
    false
}

/// Number of arguments of a `load` call that are not options (`-mat`).
fn file_argument_count(node: Node, source: &str) -> usize {
    let mut cursor = node.walk();
    let args: Vec<Node> = if node.kind() == "function_call" {
        match find_child_of_kind(node, "arguments") {
            Some(args) => args
                .children(&mut cursor)
                .filter(|c| c.is_named())
                .collect(),
            None => Vec::new(),
        }
    } else {
        node.children(&mut cursor)
            .filter(|c| c.is_named())
            .skip(1)
            .collect()
    };
    args.iter()
        .filter(|a| {
            !node_text(**a, source)
                .trim_matches(['\'', '"'])
                .starts_with('-')
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_silent_at_script_level() {
        let source = "load('data.mat')\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let fc = find_child_of_kind(root, "function_call").unwrap();
        assert!(eng.check_load(fc, source).is_empty());
    }

    #[test]
    fn test_load_silent_with_variable_list() {
        let source = "function f()\nload('data.mat', 'x')\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let fc = find_descendant_of_kind(root, "function_call").unwrap();
        assert!(eng.check_load(fc, source).is_empty());
    }

    /// Number of LOAD diagnostics for `src`, over every call and command node.
    fn load_count(src: &str) -> usize {
        let tree = parse(src);
        let eng = engine();
        let mut count = 0;
        let mut stack = vec![tree.root_node()];
        while let Some(node) = stack.pop() {
            if matches!(node.kind(), "function_call" | "command") {
                count += eng.check_load(node, src).len();
            }
            let mut cursor = node.walk();
            stack.extend(node.children(&mut cursor));
        }
        count
    }

    #[test]
    fn test_load_fires_in_anonymous_function_in_function() {
        // Probed: `h = @() load('x.mat');` inside a function reports LOAD.
        assert_eq!(load_count("function f()\nh = @() load('x.mat');\nend\n"), 1);
    }

    #[test]
    fn test_load_fires_in_anonymous_function_as_argument() {
        // Probed: the parallel.pool.Constant case and cellfun(@(q) load(q), ...).
        assert_eq!(
            load_count(
                "function c = f()\nc = parallel.pool.Constant(@() load('calibration.mat'));\nend\n"
            ),
            1
        );
        assert_eq!(
            load_count("function out = f(files)\nout = cellfun(@(q) load(q), files, 'UniformOutput', false);\nend\n"),
            1
        );
    }

    #[test]
    fn test_load_silent_in_anonymous_function_with_variable_list() {
        // Probed: naming the variables silences LOAD, also inside an anonymous function.
        assert_eq!(
            load_count("function h = f()\nh = @() load('x.mat', 'v');\nend\n"),
            0
        );
    }

    #[test]
    fn test_load_silent_in_anonymous_function_at_script_level() {
        // Probed: scripts are silent, anonymous functions included.
        assert_eq!(load_count("h = @() load('x.mat');\n"), 0);
    }

    #[test]
    fn test_load_fires_in_function_with_option_only() {
        let source = "function f()\nload('data.mat', '-mat')\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let fc = find_descendant_of_kind(root, "function_call").unwrap();
        assert_eq!(eng.check_load(fc, source).len(), 1);
    }

    #[test]
    fn test_load_fires_without_output() {
        let source = "function f()\nload('data.mat');\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let fc = find_descendant_of_kind(root, "function_call").unwrap();
        let diags = eng.check_load(fc, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "LOAD");
    }

    #[test]
    fn test_load_silent_with_output() {
        let source = "s = load('data.mat');\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        // The function_call is inside an assignment, not at statement level.
        let assignment = find_child_of_kind(root, "assignment").unwrap();
        let fc = find_child_of_kind(assignment, "function_call");
        if let Some(fc) = fc {
            let diags = eng.check_load(fc, source);
            assert!(diags.is_empty());
        }
    }
}
