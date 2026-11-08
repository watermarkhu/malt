//! Lightweight type inference for lint rules.
//!
//! MATLAB is dynamically typed, so full type inference is impossible. This
//! module provides a deliberately conservative, one-pass approximation that
//! records *provable* facts about variables: the kind of value they hold
//! (logical, numeric, char, string, cell, struct, handle, function handle) and
//! whether they are known to be scalar. Anything that cannot be proven stays
//! [`TypeInfo::Unknown`], and rules using this module only fire on provable
//! cases — the same "might" semantics MATLAB's Code Analyzer uses.
//!
//! The [`TypeEnv`] is built with a single tree-sitter DFS in
//! [`TypeEnv::build`]. It is consumed by the Good Practices logical-usage
//! checks (BDLGI, BDLOG1, BDLOG2, BDSCA, BDSCI) and the handle-default checks
//! (MCHDP, MCHDT).

use std::collections::HashMap;

use tree_sitter::{Node, Tree};

use crate::analysis::metadata::FileMeta;

/// The inferred kind of a value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeKind {
    /// `true` / `false` or the result of a comparison / boolean operator.
    Logical,
    /// A numeric literal or arithmetic result.
    Numeric,
    /// A single-quoted character vector (`'abc'`).
    Char,
    /// A double-quoted string (`"abc"`).
    String,
    /// A cell array literal (`{...}`).
    Cell,
    /// A struct or struct-array literal.
    Struct,
    /// An instance of a handle class (or a handle-valued default).
    Handle,
    /// An anonymous function (`@(...) ...`).
    FunctionHandle,
    /// Not determinable from the source.
    Unknown,
}

/// The inferred type of a variable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeInfo {
    /// The value kind.
    pub kind: TypeKind,
    /// Whether the value is known to be scalar (size `1x1`).
    pub scalar: bool,
}

impl TypeInfo {
    /// A logical scalar (`true` / `false`).
    pub fn logical_scalar() -> Self {
        Self {
            kind: TypeKind::Logical,
            scalar: true,
        }
    }

    /// A value of the given kind whose scalar-ness is unknown.
    pub fn of(kind: TypeKind) -> Self {
        Self {
            kind,
            scalar: false,
        }
    }
}

/// The result of `TypeEnv::build`: a map from variable name to its inferred
/// type. The map is file-wide and last-assignment-wins; cross-scope nuance is
/// intentionally ignored for these conservative checks.
#[derive(Debug, Clone, Default)]
pub struct TypeEnv {
    /// Variable name → inferred type.
    pub types: HashMap<String, TypeInfo>,
}

impl TypeEnv {
    /// Build a type environment by walking the tree once.
    pub fn build(tree: &Tree, source: &str) -> Self {
        let meta = FileMeta::build(tree, source);
        let mut env = Self::default();
        let mut walker = Walker {
            source,
            env: &mut env,
            meta: &meta,
        };
        walker.walk(tree.root_node());
        env
    }

    /// The inferred type of `name`, or [`TypeInfo::of(TypeKind::Unknown)`].
    pub fn type_of(&self, name: &str) -> TypeInfo {
        self.types
            .get(name)
            .copied()
            .unwrap_or_else(|| TypeInfo::of(TypeKind::Unknown))
    }

    /// Whether `name` is known to be a logical scalar.
    pub fn is_logical_scalar(&self, name: &str) -> bool {
        let t = self.type_of(name);
        t.kind == TypeKind::Logical && t.scalar
    }

    /// Whether `name` is known to be logical (scalar or not).
    pub fn is_logical(&self, name: &str) -> bool {
        self.type_of(name).kind == TypeKind::Logical
    }
}

// ---------------------------------------------------------------------------
// Walker (private)
// ---------------------------------------------------------------------------

struct Walker<'a> {
    source: &'a str,
    env: &'a mut TypeEnv,
    meta: &'a FileMeta,
}

impl<'a> Walker<'a> {
    fn text(&self, node: Node) -> &'a str {
        &self.source[node.start_byte()..node.end_byte()]
    }

    /// Infer the type of an expression node (best effort).
    fn infer_expr(&self, node: Node) -> Option<TypeInfo> {
        match node.kind() {
            "true" => Some(TypeInfo::logical_scalar()),
            "false" => Some(TypeInfo::logical_scalar()),
            "number" => Some(TypeInfo::of(TypeKind::Numeric)),
            "string" => {
                let t = self.text(node);
                if t.starts_with('"') {
                    Some(TypeInfo::of(TypeKind::String))
                } else {
                    Some(TypeInfo::of(TypeKind::Char))
                }
            }
            "cell" => Some(TypeInfo::of(TypeKind::Cell)),
            "matrix" => Some(TypeInfo::of(TypeKind::Numeric)), // could be char, but conservative
            "identifier" => {
                let name = self.text(node);
                match name {
                    "true" | "false" => Some(TypeInfo::logical_scalar()),
                    "pi" | "Inf" | "inf" | "NaN" | "nan" | "eps" | "i" | "j" => {
                        Some(TypeInfo::of(TypeKind::Numeric))
                    }
                    _ => Some(self.env.type_of(name)),
                }
            }
            "lambda" => Some(TypeInfo::of(TypeKind::FunctionHandle)),
            "metaclass_operator" => Some(TypeInfo::of(TypeKind::Handle)),
            "binary_operator"
            | "comparison_operator"
            | "boolean_operator"
            | "boolean_operator_short"
            | "unary_operator"
            | "postfix_operator" => self.infer_operator(node),
            "parenthesis" | "range" => {
                // A parenthesized expression inherits its child's type; a range
                // is numeric.
                if node.kind() == "range" {
                    return Some(TypeInfo::of(TypeKind::Numeric));
                }
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.is_named() {
                        return self.infer_expr(child);
                    }
                }
                None
            }
            "function_call" | "command" => self.infer_call(node),
            _ => None,
        }
    }

    /// Infer the type produced by an operator expression.
    fn infer_operator(&self, node: Node) -> Option<TypeInfo> {
        let mut operands: Vec<TypeInfo> = Vec::new();
        let mut scalar = true;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if !child.is_named() {
                continue;
            }
            match child.kind() {
                "=" | "~=" | "<" | ">" | "<=" | ">=" | "==" => {}
                _ => {}
            }
            if child.kind() == "=" {
                continue;
            }
            if let Some(t) = self.infer_expr(child) {
                scalar = scalar && t.scalar;
                operands.push(t);
            } else {
                scalar = false;
            }
        }
        let kind = match node.kind() {
            "comparison_operator" => TypeKind::Logical,
            "boolean_operator" | "boolean_operator_short" => TypeKind::Logical,
            "unary_operator" => {
                // `~x` is logical; `-x`/`+x` are numeric.
                if self.text(node).contains('~') {
                    TypeKind::Logical
                } else {
                    TypeKind::Numeric
                }
            }
            "postfix_operator" => operands
                .first()
                .map(|o| o.kind)
                .unwrap_or(TypeKind::Numeric),
            _ => {
                // Arithmetic `+ - * / ^ .* ./ .^ \` are numeric.
                let txt = self.text(node);
                let op = txt
                    .chars()
                    .find(|c| "+-*/^\\&|<>=~.@".contains(*c))
                    .unwrap_or('+');
                match op {
                    '&' | '|' | '<' | '>' | '~' | '=' => TypeKind::Logical,
                    _ => TypeKind::Numeric,
                }
            }
        };
        Some(TypeInfo { kind, scalar })
    }

    /// Infer the type of a function call, using metadata where the call
    /// resolves to a known class/function.
    fn infer_call(&self, node: Node) -> Option<TypeInfo> {
        // Extract the callee name.
        let name = match node.kind() {
            "function_call" => {
                let name_node = node.child_by_field_name("name")?;
                Some(self.text(name_node))
            }
            "command" => {
                let name_node = node.child(0)?;
                if name_node.kind() == "command_name" {
                    Some(self.text(name_node))
                } else {
                    None
                }
            }
            _ => None,
        }?;

        // Known value-returning builtins.
        match name {
            "zeros" | "ones" | "eye" | "rand" | "randn" | "randi" | "linspace" | "logspace"
            | "nan" | "inf" | "abs" | "ceil" | "floor" | "round" | "fix" | "mod" | "rem"
            | "sqrt" | "exp" | "log" | "log10" | "sin" | "cos" | "tan" | "sum" | "mean"
            | "median" | "std" | "var" | "min" | "max" | "prod" | "numel" | "length" | "size"
            | "ndims" | "rank" | "det" | "norm" | "reshape" | "repmat" | "sort" => {
                return Some(TypeInfo::of(TypeKind::Numeric));
            }
            "isempty" | "isequal" | "isnan" | "isinf" | "islogical" | "isnumeric" | "ischar"
            | "isstring" | "iscell" | "isstruct" | "isscalar" | "isvector" | "ismatrix"
            | "isrow" | "iscolumn" | "isreal" | "isa" | "isfinite" | "strcmp" | "strcmpi"
            | "any" | "all" | "exist" | "isvalid" | "isprop" => {
                return Some(TypeInfo::logical_scalar());
            }
            "char" | "sprintf" | "num2str" | "int2str" | "mat2str" | "lower" | "upper"
            | "strtrim" | "deblank" | "strcat" | "strjoin" => {
                return Some(TypeInfo::of(TypeKind::Char));
            }
            "string" | "string2" | "convertCharsToStrings" => {
                return Some(TypeInfo::of(TypeKind::String));
            }
            "struct" | "struct2cell" => return Some(TypeInfo::of(TypeKind::Struct)),
            "cell" => return Some(TypeInfo::of(TypeKind::Cell)),
            "handle" | "onCleanup" => return Some(TypeInfo::of(TypeKind::Handle)),
            _ => {}
        }

        // A call whose callee is a class name → handle (if the class is a
        // handle) or a method on a handle object.
        // A call whose callee is the file's own class name → handle if the
        // class is a handle.
        if let Some(class) = self.meta.class.as_ref() {
            if class.name == name {
                return if class.is_handle() {
                    Some(TypeInfo::of(TypeKind::Handle))
                } else {
                    Some(TypeInfo::of(TypeKind::Unknown))
                };
            }
        }
        if name.contains('.') {
            // `obj.method(...)` — treat as unknown unless we track object vars.
            return None;
        }

        None
    }

    /// Record the type of the target of an assignment.
    fn record_assignment(&mut self, node: Node) {
        // `lhs = rhs`
        let lhs = match node.child_by_field_name("left") {
            Some(l) => l,
            None => return,
        };
        let rhs = match node.child_by_field_name("right") {
            Some(r) => r,
            None => return,
        };
        // Only track simple identifiers on the LHS.
        if lhs.kind() != "identifier" {
            return;
        }
        let name = self.text(lhs).to_string();
        let info = self
            .infer_expr(rhs)
            .unwrap_or_else(|| TypeInfo::of(TypeKind::Unknown));
        self.env.types.insert(name, info);
    }

    fn walk(&mut self, node: Node) {
        match node.kind() {
            "assignment" => self.record_assignment(node),
            "global" | "persistent" => {
                // `global x` — treat as unknown so we never over-fire.
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "identifier" {
                        self.env.types.insert(
                            self.text(child).to_string(),
                            TypeInfo::of(TypeKind::Unknown),
                        );
                    }
                }
            }
            _ => {}
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
        assert_eq!(e.type_of("x").kind, TypeKind::Logical);
        assert!(e.type_of("x").scalar);
        assert_eq!(e.type_of("y").kind, TypeKind::Logical);
    }

    #[test]
    fn infers_numeric_from_literal() {
        let e = env("x = 42;\n");
        assert_eq!(e.type_of("x").kind, TypeKind::Numeric);
    }

    #[test]
    fn infers_char_from_single_quotes() {
        let e = env("x = 'hello';\n");
        assert_eq!(e.type_of("x").kind, TypeKind::Char);
    }

    #[test]
    fn infers_string_from_double_quotes() {
        let e = env("x = \"hello\";\n");
        assert_eq!(e.type_of("x").kind, TypeKind::String);
    }

    #[test]
    fn infers_logical_from_comparison() {
        let e = env("x = a > b;\n");
        assert_eq!(e.type_of("x").kind, TypeKind::Logical);
    }

    #[test]
    fn infers_numeric_from_arithmetic() {
        let e = env("x = a + b;\n");
        assert_eq!(e.type_of("x").kind, TypeKind::Numeric);
    }

    #[test]
    fn infers_handle_from_builtin() {
        let e = env("x = handle();\n");
        assert_eq!(e.type_of("x").kind, TypeKind::Handle);
    }

    #[test]
    fn unknown_stays_unknown() {
        let e = env("x = someMysteryFn();\n");
        assert_eq!(e.type_of("x").kind, TypeKind::Unknown);
    }

    #[test]
    fn logical_helpers() {
        let e = env("a = true;\nb = a > 3;\nc = [1 2 3];\n");
        assert!(e.is_logical_scalar("a"));
        assert!(e.is_logical("b"));
        assert!(!e.is_logical("c"));
        assert!(!e.is_logical_scalar("c"));
    }
}
