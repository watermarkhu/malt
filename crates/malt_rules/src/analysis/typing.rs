//! Lightweight type inference for lint rules.
//!
//! MATLAB is dynamically typed, so full type inference is impossible. This
//! module is a deliberately conservative, one-pass approximation that records
//! only *provable* facts: the kind of value a variable holds (logical, numeric,
//! char, string, cell, struct, handle, function handle).
//!
//! Confidence is explicit. A variable is either known to hold a [`TypeKind`]
//! or it has no entry at all, and [`TypeEnv::known_kind`] returns `None` for
//! the latter. A consumer must not fire on `None`: that is the "might" semantics
//! MATLAB's Code Analyzer uses. There is no "unknown" kind, so a guessed kind
//! can never be mistaken for a proven one.
//!
//! The [`TypeEnv`] is built with a single tree-sitter DFS in
//! [`TypeEnv::build`]. Its only consumer is the handle-default check MCHDT
//! (`good_practices/check_handle_defaults.rs`). The map is file-wide and
//! last-assignment-wins, on purpose: MCHDT resolves a property default through
//! an assignment made in a method, which a reaching-definitions walk over the
//! property block would not see.

use std::collections::HashMap;

use tree_sitter::{Node, Tree};

use crate::analysis::metadata::FileMeta;

/// The kind of value a variable is known to hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeKind {
    /// `true` / `false` or the result of a comparison / boolean operator.
    Logical,
    /// A numeric literal, a matrix or range, or an arithmetic result.
    Numeric,
    /// A single-quoted character vector (`'abc'`).
    Char,
    /// A double-quoted string (`"abc"`).
    String,
    /// A cell array literal (`{...}`).
    Cell,
    /// A struct or struct-array literal.
    Struct,
    /// An instance of a handle class (or a handle-valued builtin).
    Handle,
    /// An anonymous function (`@(...) ...`).
    FunctionHandle,
}

/// The result of [`TypeEnv::build`]: the variables whose kind is known.
#[derive(Debug, Clone, Default)]
pub struct TypeEnv {
    /// Variable name → the kind it is known to hold. Absent when not known.
    kinds: HashMap<String, TypeKind>,
}

impl TypeEnv {
    /// Build a type environment by walking the tree once.
    pub fn build(tree: &Tree, source: &str) -> Self {
        let meta = FileMeta::build(tree, source);
        let mut walker = Walker {
            source,
            kinds: HashMap::new(),
            meta: &meta,
        };
        walker.walk(tree.root_node());
        Self {
            kinds: walker.kinds,
        }
    }

    /// The kind `name` is known to hold, or `None` when it is not known. A
    /// consumer must not fire on `None`.
    pub fn known_kind(&self, name: &str) -> Option<TypeKind> {
        self.kinds.get(name).copied()
    }
}

// ---------------------------------------------------------------------------
// Walker (private)
// ---------------------------------------------------------------------------

struct Walker<'a> {
    source: &'a str,
    kinds: HashMap<String, TypeKind>,
    meta: &'a FileMeta,
}

impl<'a> Walker<'a> {
    fn text(&self, node: Node) -> &'a str {
        &self.source[node.start_byte()..node.end_byte()]
    }

    /// The kind of an expression, or `None` when the source does not prove it.
    fn infer_expr(&self, node: Node) -> Option<TypeKind> {
        match node.kind() {
            "true" | "false" => Some(TypeKind::Logical),
            "number" => Some(TypeKind::Numeric),
            "string" => {
                if self.text(node).starts_with('"') {
                    Some(TypeKind::String)
                } else {
                    Some(TypeKind::Char)
                }
            }
            "cell" => Some(TypeKind::Cell),
            // A matrix could be char; it is taken as numeric, as before.
            "matrix" => Some(TypeKind::Numeric),
            "identifier" => match self.text(node) {
                "true" | "false" => Some(TypeKind::Logical),
                "pi" | "Inf" | "inf" | "NaN" | "nan" | "eps" | "i" | "j" => Some(TypeKind::Numeric),
                name => self.kinds.get(name).copied(),
            },
            "lambda" => Some(TypeKind::FunctionHandle),
            "metaclass_operator" => Some(TypeKind::Handle),
            "range" => Some(TypeKind::Numeric),
            "parenthesis" => {
                // A parenthesized expression has the kind of its first named child.
                let mut cursor = node.walk();
                let first = node.children(&mut cursor).find(|c| c.is_named());
                first.and_then(|child| self.infer_expr(child))
            }
            "binary_operator"
            | "comparison_operator"
            | "boolean_operator"
            | "boolean_operator_short"
            | "unary_operator"
            | "postfix_operator" => self.infer_operator(node),
            "function_call" | "command" => self.infer_call(node),
            _ => None,
        }
    }

    /// The kind an operator expression produces. Comparisons and boolean operators
    /// are logical, `~x` is logical, and arithmetic is numeric. A transpose has the
    /// kind of its operand.
    fn infer_operator(&self, node: Node) -> Option<TypeKind> {
        match node.kind() {
            "comparison_operator" | "boolean_operator" | "boolean_operator_short" => {
                Some(TypeKind::Logical)
            }
            "unary_operator" => Some(if self.text(node).contains('~') {
                TypeKind::Logical
            } else {
                TypeKind::Numeric
            }),
            "postfix_operator" => {
                let mut cursor = node.walk();
                let operand = node
                    .named_children(&mut cursor)
                    .find_map(|child| self.infer_expr(child));
                operand
            }
            _ => {
                // Arithmetic `+ - * / ^ .* ./ .^ \` is numeric.
                let txt = self.text(node);
                let op = txt
                    .chars()
                    .find(|c| "+-*/^\\&|<>=~.@".contains(*c))
                    .unwrap_or('+');
                match op {
                    '&' | '|' | '<' | '>' | '~' | '=' => Some(TypeKind::Logical),
                    _ => Some(TypeKind::Numeric),
                }
            }
        }
    }

    /// The kind a function call produces, for a builtin with a known result kind or
    /// the file's own handle class.
    fn infer_call(&self, node: Node) -> Option<TypeKind> {
        let name = match node.kind() {
            "function_call" => self.text(node.child_by_field_name("name")?),
            "command" => {
                let name_node = node.child(0)?;
                if name_node.kind() != "command_name" {
                    return None;
                }
                self.text(name_node)
            }
            _ => return None,
        };

        match name {
            "zeros" | "ones" | "eye" | "rand" | "randn" | "randi" | "linspace" | "logspace"
            | "nan" | "inf" | "abs" | "ceil" | "floor" | "round" | "fix" | "mod" | "rem"
            | "sqrt" | "exp" | "log" | "log10" | "sin" | "cos" | "tan" | "sum" | "mean"
            | "median" | "std" | "var" | "min" | "max" | "prod" | "numel" | "length" | "size"
            | "ndims" | "rank" | "det" | "norm" | "reshape" | "repmat" | "sort" => {
                Some(TypeKind::Numeric)
            }
            "isempty" | "isequal" | "isnan" | "isinf" | "islogical" | "isnumeric" | "ischar"
            | "isstring" | "iscell" | "isstruct" | "isscalar" | "isvector" | "ismatrix"
            | "isrow" | "iscolumn" | "isreal" | "isa" | "isfinite" | "strcmp" | "strcmpi"
            | "any" | "all" | "exist" | "isvalid" | "isprop" => Some(TypeKind::Logical),
            "char" | "sprintf" | "num2str" | "int2str" | "mat2str" | "lower" | "upper"
            | "strtrim" | "deblank" | "strcat" | "strjoin" => Some(TypeKind::Char),
            "string" | "string2" | "convertCharsToStrings" => Some(TypeKind::String),
            "struct" | "struct2cell" => Some(TypeKind::Struct),
            "cell" => Some(TypeKind::Cell),
            "handle" | "onCleanup" => Some(TypeKind::Handle),
            // A call to the file's own class is a handle when the class is one. For a
            // value class the kind is not known.
            _ => self
                .meta
                .class
                .as_ref()
                .filter(|class| class.name == name && class.is_handle())
                .map(|_| TypeKind::Handle),
        }
    }

    /// Record the kind of the target of `lhs = rhs`. An assignment whose right side
    /// is not known removes the variable, so a later unknown value is not hidden by
    /// an earlier known one.
    fn record_assignment(&mut self, node: Node) {
        let (Some(lhs), Some(rhs)) = (
            node.child_by_field_name("left"),
            node.child_by_field_name("right"),
        ) else {
            return;
        };
        // Only plain identifiers on the left are tracked.
        if lhs.kind() != "identifier" {
            return;
        }
        let name = self.text(lhs).to_string();
        match self.infer_expr(rhs) {
            Some(kind) => self.kinds.insert(name, kind),
            None => self.kinds.remove(&name),
        };
    }

    fn walk(&mut self, node: Node) {
        // `global x` and `persistent x` are not handled: the grammar names them
        // `global_operator` and `persistent_operator`, and handling them would change
        // MCHDT on a `global` default, so that is left for a decision.
        if node.kind() == "assignment" {
            self.record_assignment(node);
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.walk(child);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::parse;

    fn env(source: &str) -> TypeEnv {
        let tree = parse(source);
        TypeEnv::build(&tree, source)
    }

    #[test]
    fn infers_logical_from_literal() {
        let e = env("x = true;\ny = false;\n");
        assert_eq!(e.known_kind("x"), Some(TypeKind::Logical));
        assert_eq!(e.known_kind("y"), Some(TypeKind::Logical));
    }

    #[test]
    fn infers_numeric_from_literal() {
        assert_eq!(env("x = 42;\n").known_kind("x"), Some(TypeKind::Numeric));
    }

    #[test]
    fn infers_char_from_single_quotes() {
        assert_eq!(env("x = 'hello';\n").known_kind("x"), Some(TypeKind::Char));
    }

    #[test]
    fn infers_string_from_double_quotes() {
        assert_eq!(
            env("x = \"hello\";\n").known_kind("x"),
            Some(TypeKind::String)
        );
    }

    #[test]
    fn infers_logical_from_comparison() {
        assert_eq!(env("x = a > b;\n").known_kind("x"), Some(TypeKind::Logical));
    }

    #[test]
    fn infers_numeric_from_arithmetic() {
        assert_eq!(env("x = a + b;\n").known_kind("x"), Some(TypeKind::Numeric));
    }

    #[test]
    fn infers_handle_from_builtin() {
        assert_eq!(
            env("x = handle();\n").known_kind("x"),
            Some(TypeKind::Handle)
        );
    }

    #[test]
    fn unknown_has_no_kind() {
        assert_eq!(env("x = someMysteryFn();\n").known_kind("x"), None);
        assert_eq!(env("x = y;\n").known_kind("x"), None);
    }

    #[test]
    fn later_unknown_assignment_hides_earlier_kind() {
        let e = env("x = handle();\nx = someMysteryFn();\n");
        assert_eq!(e.known_kind("x"), None);
    }

    #[test]
    fn transpose_keeps_operand_kind_and_unknown_operand_stays_unknown() {
        let e = env("h = handle();\nt = h';\nu = someFn()';\n");
        assert_eq!(e.known_kind("t"), Some(TypeKind::Handle));
        assert_eq!(e.known_kind("u"), None);
    }

    #[test]
    fn logical_and_matrix_kinds() {
        let e = env("a = true;\nb = a > 3;\nc = [1 2 3];\n");
        assert_eq!(e.known_kind("a"), Some(TypeKind::Logical));
        assert_eq!(e.known_kind("b"), Some(TypeKind::Logical));
        assert_eq!(e.known_kind("c"), Some(TypeKind::Numeric));
    }
}
