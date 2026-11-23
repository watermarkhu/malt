//! # Command-syntax nodes
//!
//! A `command` node is command syntax (`hold on`, `format long`, `!ls -la`), or an
//! expression the grammar reads as one. `comm.gpu.AWGNChannel arg` parses as a command
//! whose name is `comm` and whose first argument is `.gpu.AWGNChannel`, glued to the
//! name. MATLAB reports that as a syntax error (SYNER), so it is not a call of `comm`.
//!
//! [`Command::from_node`] reads the name and the arguments of a `command` node once,
//! and [`Command::is_command`] answers whether the node is a real command.

use tree_sitter::Node;

/// A `command` node, read once.
#[derive(Debug, Clone)]
pub struct Command<'a> {
    /// The `command` node.
    pub node: Node<'a>,
    /// The name as written: `hold`, or `!ls` for a shell escape.
    pub name: &'a str,
    /// The `command_argument` children, in order. Comments and continuations are not
    /// arguments.
    pub argument_nodes: Vec<Node<'a>>,
    /// The text of each argument with one pair of matching quotes removed: `'a b'`
    /// reads as `a b`. Doubled quotes inside are kept as written (`'it''s'` is `it''s`).
    pub arguments: Vec<&'a str>,
    /// The second argument when it is glued to the name and starts with `.`: the
    /// dotted name the grammar split (`.gpu.AWGNChannel` in `comm.gpu.AWGNChannel arg`).
    glued_dotted: Option<&'a str>,
}

impl<'a> Command<'a> {
    /// Read a `command` node. `None` for any other node, or a command with no
    /// `command_name` first.
    pub fn from_node(node: Node<'a>, source: &'a str) -> Option<Self> {
        if node.kind() != "command" {
            return None;
        }
        let name_node = node.child(0).filter(|n| n.kind() == "command_name")?;
        let name = &source[name_node.byte_range()];

        let mut argument_nodes = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "command_argument" {
                argument_nodes.push(child);
            }
        }
        let arguments = argument_nodes
            .iter()
            .map(|arg| unquote(&source[arg.byte_range()]))
            .collect();

        // Only the second child can be the glued dotted part: a word glued to the name
        // is the next child, with no space between.
        let glued_dotted = node
            .child(1)
            .filter(|arg| {
                arg.kind() == "command_argument"
                    && arg.start_byte() == name_node.end_byte()
                    && source[arg.start_byte()..].starts_with('.')
            })
            .map(|arg| &source[arg.byte_range()]);

        Some(Self {
            node,
            name,
            argument_nodes,
            arguments,
            glued_dotted,
        })
    }

    /// Whether the name is a shell escape: `!ls -la`.
    pub fn is_shell_escape(&self) -> bool {
        self.name.starts_with('!')
    }

    /// Whether the name is a plain identifier (letters, digits and `_`, not starting
    /// with a digit).
    pub fn has_identifier_name(&self) -> bool {
        let mut chars = self.name.chars();
        chars.next().is_some_and(|c| c.is_alphabetic() || c == '_')
            && chars.all(|c| c.is_alphanumeric() || c == '_')
    }

    /// Whether the node is a dotted name the grammar split into a command: the name
    /// is glued to a following `.part` argument (`comm.gpu.AWGNChannel arg`).
    pub fn is_dotted_misread(&self) -> bool {
        self.glued_dotted.is_some()
    }

    /// The full dotted name of a misread, `comm.gpu.AWGNChannel` for the example in
    /// [`Command::is_dotted_misread`]. `None` for a real command.
    pub fn dotted_name(&self) -> Option<String> {
        self.glued_dotted
            .map(|glued| format!("{}{}", self.name, glued))
    }

    /// Whether the node is a real command: its name is a plain identifier or a shell
    /// escape, and it is not a dotted name split by the grammar.
    pub fn is_command(&self) -> bool {
        !self.is_dotted_misread() && (self.is_shell_escape() || self.has_identifier_name())
    }
}

/// The text inside one pair of matching quotes, or the text itself.
fn unquote(text: &str) -> &str {
    for quote in ['\'', '"'] {
        if let Some(inner) = text
            .strip_prefix(quote)
            .and_then(|rest| rest.strip_suffix(quote))
        {
            return inner;
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Parser;

    /// The first `command` node of `source`, read as a [`Command`].
    fn command_of<'a>(tree: &'a tree_sitter::Tree, source: &'a str) -> Option<Command<'a>> {
        let mut stack = vec![tree.root_node()];
        while let Some(node) = stack.pop() {
            if node.kind() == "command" {
                return Command::from_node(node, source);
            }
            let mut cursor = node.walk();
            stack.extend(node.children(&mut cursor));
        }
        None
    }

    fn parse(source: &str) -> tree_sitter::Tree {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_matlab::LANGUAGE.into())
            .expect("failed to set MATLAB language");
        malt_core::note_parse();
        parser.parse(source, None).expect("failed to parse")
    }

    #[test]
    fn reads_name_and_unquoted_arguments() {
        let source = "disp 'a b' x\n";
        let tree = parse(source);
        let cmd = command_of(&tree, source).unwrap();
        assert_eq!(cmd.name, "disp");
        assert_eq!(cmd.arguments, vec!["a b", "x"]);
        assert_eq!(cmd.argument_nodes.len(), 2);
        assert!(cmd.is_command());
        assert!(!cmd.is_dotted_misread());
        assert_eq!(cmd.dotted_name(), None);
    }

    #[test]
    fn continuations_and_comments_are_not_arguments() {
        let source = "disp x ...\n  y % tail\n";
        let tree = parse(source);
        let cmd = command_of(&tree, source).unwrap();
        assert_eq!(cmd.arguments, vec!["x", "y"]);
    }

    #[test]
    fn shell_escape_is_a_command() {
        let source = "!ls -la\n";
        let tree = parse(source);
        let cmd = command_of(&tree, source).unwrap();
        assert_eq!(cmd.name, "!ls");
        assert!(cmd.is_shell_escape());
        assert!(cmd.is_command());
    }

    #[test]
    fn dotted_misread_is_not_a_command() {
        let source = "comm.gpu.AWGNChannel arg\n";
        let tree = parse(source);
        let cmd = command_of(&tree, source).unwrap();
        assert_eq!(cmd.name, "comm");
        assert!(cmd.is_dotted_misread());
        assert_eq!(cmd.dotted_name().as_deref(), Some("comm.gpu.AWGNChannel"));
        assert!(!cmd.is_command());
    }

    #[test]
    fn spaced_argument_is_a_real_command() {
        // `a .b arg` has a space before `.b`: not glued, so not a dotted name.
        let source = "a .b arg\n";
        let tree = parse(source);
        let cmd = command_of(&tree, source).unwrap();
        assert!(!cmd.is_dotted_misread());
    }

    #[test]
    fn bare_name_is_a_command_and_expressions_are_not_read() {
        let source = "hold on\n";
        let tree = parse(source);
        assert_eq!(command_of(&tree, source).unwrap().name, "hold");

        let source = "x = 1;\n";
        let tree = parse(source);
        assert!(command_of(&tree, source).is_none());
    }
}
