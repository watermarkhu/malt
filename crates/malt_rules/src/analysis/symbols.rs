//! # Symbol Table — Per-scope variable definition and usage tracking
//!
//! This module provides a [`SymbolTable`] that maps out every variable definition
//! and usage across all scopes in a MATLAB source file. It is the foundation for
//! rules that need scope-aware analysis such as unused variables, undefined
//! variables, and shadowed variables.
//!
//! ## How it works
//!
//! [`SymbolTable::build`] performs a single depth-first traversal of the
//! tree-sitter parse tree. It maintains a scope stack: every time it enters a
//! `function_definition` or `lambda`, a new [`Scope`] is pushed. When the
//! traversal leaves that node, the scope is popped.
//!
//! For each identifier encountered, the builder inspects the parent node to
//! decide whether the identifier is a **definition** ([`VarDef`]) or a
//! **usage** ([`VarUse`]). Definitions include input/output arguments,
//! assignment left-hand sides, for-loop iterators, `global`/`persistent`
//! declarations, nested function names, and lambda parameters. Everything
//! else is a usage.
//!
//! ## Example
//!
//! ```ignore
//! use tree_sitter::Parser;
//! use malt_rules::analysis::symbols::SymbolTable;
//!
//! let source = "function y = foo(x)\n    y = x + 1;\nend\n";
//! let mut parser = Parser::new();
//! parser.set_language(&tree_sitter_matlab::LANGUAGE.into()).unwrap();
//! let tree = parser.parse(source, None).unwrap();
//!
//! let table = SymbolTable::build(&tree, source);
//! assert_eq!(table.scopes.len(), 1);
//! assert!(table.root_scope().is_defined("x"));
//! assert!(table.root_scope().is_defined("y"));
//! ```

use std::collections::HashSet;
use tree_sitter::Tree;

/// Whether `node` is the name of a field in `a.b.c` (anything but the leading
/// object), as opposed to a variable. The expression in a dynamic field
/// `s.(name)` sits inside an `indirect_access` node and is a variable.
pub(crate) fn is_field_name(node: tree_sitter::Node) -> bool {
    node.parent().is_some_and(|p| {
        p.kind() == "field_expression"
            && p.named_child(0)
                .is_some_and(|object| object.id() != node.id())
    })
}

// ---------------------------------------------------------------------------
// Definition kinds
// ---------------------------------------------------------------------------

/// The type of a variable definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefKind {
    /// Function input argument.
    InputArg,
    /// Function output argument.
    OutputArg,
    /// Assignment left-hand side (`x = ...`).
    Assignment,
    /// For-loop iterator variable.
    ForIterator,
    /// Global declaration (`global x`).
    Global,
    /// Persistent declaration (`persistent x`).
    Persistent,
    /// Nested function name.
    NestedFunction,
    /// Anonymous function parameter.
    LambdaParam,
}

// ---------------------------------------------------------------------------
// VarDef / VarUse
// ---------------------------------------------------------------------------

/// A single variable definition occurrence.
#[derive(Debug, Clone)]
pub struct VarDef {
    /// The variable name.
    pub name: String,
    /// How it was defined.
    pub kind: DefKind,
    /// Byte range in source.
    pub byte_range: std::ops::Range<usize>,
    /// 1-indexed line number.
    pub line: usize,
    /// 1-indexed column number.
    pub column: usize,
    /// Offset from which the definition is in effect. For an assignment this is
    /// the end of the statement: the right-hand side is evaluated first, so a
    /// use inside `x = x + 1` comes before the definition it belongs to. For
    /// every other kind it is the start of `byte_range`.
    pub effect_start: usize,
    /// A partial assignment (`A(i) = v`, `s.f = v`, `C{i} = v`) rather than a
    /// full one.
    pub partial: bool,
    /// One element of a `[a, b] = f()` output list.
    pub multi_output: bool,
    /// The right-hand side is a preallocation call (`zeros(...)`, `cell(...)`, ...).
    pub prealloc: bool,
    /// Start of the statement block the assignment sits in; two assignments with
    /// the same value run one after the other, unconditionally.
    pub block_start: usize,
}

/// A single variable usage (read) occurrence.
#[derive(Debug, Clone)]
pub struct VarUse {
    /// The variable name.
    pub name: String,
    /// Byte range in source.
    pub byte_range: std::ops::Range<usize>,
    /// 1-indexed line number.
    pub line: usize,
    /// 1-indexed column number.
    pub column: usize,
    /// The name is the callee of `name(...)`. MATLAB resolves it as an indexed
    /// variable only if a variable of that name exists; with no definition in
    /// sight it is a function call, not an undefined variable.
    pub is_call_target: bool,
}

// ---------------------------------------------------------------------------
// Scope
// ---------------------------------------------------------------------------

/// The type of scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    /// File-level script scope (no enclosing function).
    Script,
    /// Main function (first `function_definition` in file).
    Function,
    /// Local function (subsequent `function_definition`s at file level).
    LocalFunction,
    /// Nested function (`function_definition` inside another function).
    NestedFunction,
    /// Method (`function_definition` inside a `methods` block in a class).
    Method,
    /// Anonymous function body.
    Lambda,
}

/// A scope representing a function, script, or lambda body.
#[derive(Debug, Clone)]
pub struct Scope {
    /// The name of the function/method (empty string for scripts/lambdas).
    pub name: String,
    /// The kind of scope.
    pub kind: ScopeKind,
    /// Byte range of the entire scope node.
    pub byte_range: std::ops::Range<usize>,
    /// 1-indexed line of scope start.
    pub line: usize,
    /// All variable definitions in this scope.
    pub defs: Vec<VarDef>,
    /// All variable usages (reads) in this scope.
    pub uses: Vec<VarUse>,
    /// Child scope indices (nested functions, lambdas).
    pub children: Vec<usize>,
    /// Parent scope index (`None` for top-level).
    pub parent: Option<usize>,
}

impl Scope {
    /// Get all definitions of a variable by name.
    pub fn defs_of(&self, name: &str) -> Vec<&VarDef> {
        self.defs.iter().filter(|d| d.name == name).collect()
    }

    /// Get all usages of a variable by name.
    pub fn uses_of(&self, name: &str) -> Vec<&VarUse> {
        self.uses.iter().filter(|u| u.name == name).collect()
    }

    /// Get all unique defined variable names.
    pub fn defined_names(&self) -> Vec<&str> {
        let mut seen = HashSet::new();
        self.defs
            .iter()
            .filter_map(|d| {
                if seen.insert(d.name.as_str()) {
                    Some(d.name.as_str())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all unique used variable names.
    pub fn used_names(&self) -> Vec<&str> {
        let mut seen = HashSet::new();
        self.uses
            .iter()
            .filter_map(|u| {
                if seen.insert(u.name.as_str()) {
                    Some(u.name.as_str())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Check if a variable is defined in this scope.
    pub fn is_defined(&self, name: &str) -> bool {
        self.defs.iter().any(|d| d.name == name)
    }

    /// Check if a variable is used in this scope.
    pub fn is_used(&self, name: &str) -> bool {
        self.uses.iter().any(|u| u.name == name)
    }
}

// ---------------------------------------------------------------------------
// SymbolTable
// ---------------------------------------------------------------------------

/// The complete symbol table for a file.
///
/// Contains all scopes (functions, scripts, lambdas) in the file, each with
/// their own definitions and usages. Scopes are stored in a flat `Vec` and
/// reference each other by index via [`Scope::children`] and [`Scope::parent`].
#[derive(Debug, Clone)]
pub struct SymbolTable {
    /// All scopes in the file, indexed by position in this vec.
    pub scopes: Vec<Scope>,
}

impl SymbolTable {
    /// Get the root/file-level scope (index 0).
    pub fn root_scope(&self) -> &Scope {
        &self.scopes[0]
    }

    /// Get all function-level scopes (excluding lambdas and the root script scope).
    pub fn function_scopes(&self) -> Vec<&Scope> {
        self.scopes
            .iter()
            .filter(|s| {
                matches!(
                    s.kind,
                    ScopeKind::Function
                        | ScopeKind::LocalFunction
                        | ScopeKind::NestedFunction
                        | ScopeKind::Method
                )
            })
            .collect()
    }

    /// Find the innermost scope containing a given byte offset.
    ///
    /// Searches all scopes and returns the smallest (innermost) one whose
    /// byte range contains `byte_offset`.
    pub fn scope_at(&self, byte_offset: usize) -> Option<&Scope> {
        self.scopes
            .iter()
            .filter(|s| s.byte_range.contains(&byte_offset))
            .min_by_key(|s| s.byte_range.end - s.byte_range.start)
    }

    /// Build a symbol table from a parsed tree and source text.
    ///
    /// Performs a single DFS traversal of the tree-sitter parse tree, tracking
    /// scopes and collecting variable definitions and usages.
    pub fn build(tree: &Tree, source: &str) -> Self {
        let mut builder = Builder {
            source,
            scopes: Vec::new(),
            scope_stack: Vec::new(),
            first_file_function_seen: false,
        };
        builder.run(tree);
        SymbolTable {
            scopes: builder.scopes,
        }
    }
}

// ---------------------------------------------------------------------------
// Builder (private)
// ---------------------------------------------------------------------------

/// Internal builder that drives the DFS traversal and populates scopes.
struct Builder<'a> {
    source: &'a str,
    scopes: Vec<Scope>,
    /// Stack of scope indices: the last element is the "current" scope.
    scope_stack: Vec<usize>,
    /// Whether we have seen the first file-level `function_definition`.
    first_file_function_seen: bool,
}

impl<'a> Builder<'a> {
    // -- scope helpers ------------------------------------------------------

    /// Push a new scope and return its index.
    fn push_scope(&mut self, scope: Scope) -> usize {
        let idx = self.scopes.len();
        // Wire up parent ↔ child.
        let parent = self.scope_stack.last().copied();
        let mut scope = scope;
        scope.parent = parent;
        if let Some(p) = parent {
            self.scopes[p].children.push(idx);
        }
        self.scopes.push(scope);
        self.scope_stack.push(idx);
        idx
    }

    /// Pop the current scope off the stack.
    fn pop_scope(&mut self) {
        self.scope_stack.pop();
    }

    /// Index of the current scope, if any.
    fn current_scope(&self) -> Option<usize> {
        self.scope_stack.last().copied()
    }

    /// Add a definition to the current scope.
    fn add_def(&mut self, def: VarDef) {
        if let Some(idx) = self.current_scope() {
            self.scopes[idx].defs.push(def);
        }
    }

    /// Add a usage to the current scope.
    fn add_use(&mut self, var_use: VarUse) {
        if let Some(idx) = self.current_scope() {
            self.scopes[idx].uses.push(var_use);
        }
    }

    /// Extract the text of a node from source.
    fn node_text(&self, node: tree_sitter::Node) -> &'a str {
        &self.source[node.start_byte()..node.end_byte()]
    }

    // -- main entry ---------------------------------------------------------

    /// Run the full build.
    fn run(&mut self, tree: &'a Tree) {
        let root = tree.root_node();

        // Determine if this is a script or function file.
        // A function file starts with a `function_definition` as the first
        // meaningful child of `source_file`. Otherwise it's a script.
        let is_function_file = Self::first_meaningful_child_is_function(&root);

        if !is_function_file {
            // Script file: the entire file is one Script scope.
            let pos = root.start_position();
            self.push_scope(Scope {
                name: String::new(),
                kind: ScopeKind::Script,
                byte_range: root.start_byte()..root.end_byte(),
                line: pos.row + 1,
                defs: Vec::new(),
                uses: Vec::new(),
                children: Vec::new(),
                parent: None,
            });
        }

        // Walk children of source_file.
        self.visit_children(root);

        // If this was a function file and we never pushed a scope, that
        // shouldn't happen, but guard against it.
        if self.scopes.is_empty() {
            let pos = root.start_position();
            self.push_scope(Scope {
                name: String::new(),
                kind: ScopeKind::Script,
                byte_range: root.start_byte()..root.end_byte(),
                line: pos.row + 1,
                defs: Vec::new(),
                uses: Vec::new(),
                children: Vec::new(),
                parent: None,
            });
        }
    }

    /// Check if the first non-comment child of `source_file` is a function.
    fn first_meaningful_child_is_function(root: &tree_sitter::Node) -> bool {
        let count = root.child_count();
        for i in 0..count {
            if let Some(child) = root.child(i) {
                match child.kind() {
                    "comment" | "line_continuation" => continue,
                    // A class file is not a script: its properties, attributes
                    // and method names are declarations, not variable uses.
                    "function_definition" | "class_definition" => return true,
                    _ => return false,
                }
            }
        }
        false
    }

    // -- recursive visitor --------------------------------------------------

    /// Visit all children of a node, dispatching each to the appropriate handler.
    fn visit_children(&mut self, node: tree_sitter::Node<'a>) {
        let count = node.child_count();
        for i in 0..count {
            if let Some(child) = node.child(i) {
                self.visit_node(child);
            }
        }
    }

    /// Dispatch a single node.
    fn visit_node(&mut self, node: tree_sitter::Node<'a>) {
        match node.kind() {
            "function_definition" => self.visit_function_definition(node),
            "lambda" => self.visit_lambda(node),
            "assignment" => self.visit_assignment(node),
            "for_statement" => self.visit_for_statement(node),
            "global_operator" => self.visit_global_persistent(node, DefKind::Global),
            "persistent_operator" => self.visit_global_persistent(node, DefKind::Persistent),
            "identifier" => self.visit_identifier(node),
            // Class declarations: names, attributes and superclasses are not
            // reads; only property default values and method bodies are code.
            "attributes" | "superclasses" | "enum" | "events" => {}
            // `catch err` defines the exception variable.
            "catch_clause" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "identifier"
                        && child.prev_sibling().is_some_and(|p| p.kind() == "catch")
                    {
                        let pos = child.start_position();
                        let def = VarDef {
                            name: self.node_text(child).to_string(),
                            kind: DefKind::Assignment,
                            byte_range: child.byte_range(),
                            effect_start: child.end_byte(),
                            partial: false,
                            multi_output: false,
                            prealloc: false,
                            block_start: 0,
                            line: pos.row + 1,
                            column: pos.column + 1,
                        };
                        self.add_def(def);
                    } else {
                        self.visit_node(child);
                    }
                }
            }
            "property" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "default_value" {
                        self.visit_children(child);
                    }
                }
            }
            "class_definition" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() != "identifier" {
                        self.visit_node(child);
                    }
                }
            }
            // For all other nodes, just recurse into children.
            _ => self.visit_children(node),
        }
    }

    // -- function_definition ------------------------------------------------

    /// Visit a `function_definition` node.
    ///
    /// Creates a new scope, extracts input/output args, then recurses into
    /// the function body. Nested functions and methods are distinguished by
    /// the current scope stack depth and parent node context.
    fn visit_function_definition(&mut self, node: tree_sitter::Node<'a>) {
        let pos = node.start_position();

        // Extract function name.
        let func_name = node
            .child_by_field_name("name")
            .map(|n| self.node_text(n).to_string())
            .unwrap_or_default();

        // Determine scope kind.
        let scope_kind = self.classify_function_scope(node);

        // If this is a nested function, register its name as a def in the
        // parent scope (so the parent can "call" it).
        if scope_kind == ScopeKind::NestedFunction {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name_pos = name_node.start_position();
                self.add_def(VarDef {
                    name: func_name.clone(),
                    kind: DefKind::NestedFunction,
                    byte_range: name_node.start_byte()..name_node.end_byte(),
                    effect_start: name_node.start_byte(),
                    partial: false,
                    multi_output: false,
                    prealloc: false,
                    block_start: 0,
                    line: name_pos.row + 1,
                    column: name_pos.column + 1,
                });
            }
        }

        // Push new scope.
        self.push_scope(Scope {
            name: func_name,
            kind: scope_kind,
            byte_range: node.start_byte()..node.end_byte(),
            line: pos.row + 1,
            defs: Vec::new(),
            uses: Vec::new(),
            children: Vec::new(),
            parent: None, // set by push_scope
        });

        // Extract output arguments.
        self.extract_function_outputs(node);

        // Extract input arguments.
        self.extract_function_inputs(node);

        // Recurse into the function body (all children except the ones we
        // already handled).
        let count = node.child_count();
        for i in 0..count {
            if let Some(child) = node.child(i) {
                let kind = child.kind();
                if kind == "function_arguments" || kind == "function_output" {
                    // Already handled above.
                    continue;
                }
                if kind == "identifier" {
                    // Skip the function name identifier (it's the `name` field).
                    if node
                        .child_by_field_name("name")
                        .map(|n| n.id() == child.id())
                        .unwrap_or(false)
                    {
                        continue;
                    }
                }
                self.visit_node(child);
            }
        }

        self.pop_scope();
    }

    /// Classify a `function_definition` based on its position in the tree.
    fn classify_function_scope(&mut self, node: tree_sitter::Node) -> ScopeKind {
        // Check if we're inside a `methods` block.
        if Self::is_inside_methods_block(node) {
            return ScopeKind::Method;
        }

        // Check if we're inside another function scope.
        if self
            .current_scope()
            .map(|idx| {
                matches!(
                    self.scopes[idx].kind,
                    ScopeKind::Function
                        | ScopeKind::LocalFunction
                        | ScopeKind::NestedFunction
                        | ScopeKind::Method
                )
            })
            .unwrap_or(false)
        {
            return ScopeKind::NestedFunction;
        }

        // File-level function.
        if !self.first_file_function_seen {
            self.first_file_function_seen = true;
            ScopeKind::Function
        } else {
            ScopeKind::LocalFunction
        }
    }

    /// Check if a node is inside a `methods` block (by walking parents).
    fn is_inside_methods_block(node: tree_sitter::Node) -> bool {
        let mut current = node.parent();
        while let Some(p) = current {
            if p.kind() == "methods" {
                return true;
            }
            // Stop at class_definition level — don't walk further up.
            if p.kind() == "class_definition" || p.kind() == "source_file" {
                break;
            }
            current = p.parent();
        }
        false
    }

    /// Extract output arguments from `function_output`.
    fn extract_function_outputs(&mut self, func_node: tree_sitter::Node<'a>) {
        let Some(output_node) = func_node.child_by_field_name("output") else {
            // Also try iterating children for "function_output".
            let count = func_node.child_count();
            for i in 0..count {
                if let Some(child) = func_node.child(i) {
                    if child.kind() == "function_output" {
                        self.extract_output_identifiers(child);
                        return;
                    }
                }
            }
            return;
        };
        self.extract_output_identifiers(output_node);
    }

    /// Recursively extract identifiers from an output node.
    fn extract_output_identifiers(&mut self, node: tree_sitter::Node<'a>) {
        match node.kind() {
            "identifier" => {
                let pos = node.start_position();
                self.add_def(VarDef {
                    name: self.node_text(node).to_string(),
                    kind: DefKind::OutputArg,
                    byte_range: node.start_byte()..node.end_byte(),
                    effect_start: node.start_byte(),
                    partial: false,
                    multi_output: false,
                    prealloc: false,
                    block_start: 0,
                    line: pos.row + 1,
                    column: pos.column + 1,
                });
            }
            "multioutput_variable" | "function_output" => {
                let count = node.child_count();
                for i in 0..count {
                    if let Some(child) = node.child(i) {
                        if child.kind() == "identifier" {
                            let pos = child.start_position();
                            self.add_def(VarDef {
                                name: self.node_text(child).to_string(),
                                kind: DefKind::OutputArg,
                                byte_range: child.start_byte()..child.end_byte(),
                                effect_start: child.start_byte(),
                                partial: false,
                                multi_output: false,
                                prealloc: false,
                                block_start: 0,
                                line: pos.row + 1,
                                column: pos.column + 1,
                            });
                        } else if child.kind() == "multioutput_variable" {
                            self.extract_output_identifiers(child);
                        }
                        // Skip `ignored_argument` (~).
                    }
                }
            }
            _ => {}
        }
    }

    /// Extract input arguments from `function_arguments`.
    fn extract_function_inputs(&mut self, func_node: tree_sitter::Node<'a>) {
        let count = func_node.child_count();
        for i in 0..count {
            if let Some(child) = func_node.child(i) {
                if child.kind() == "function_arguments" {
                    self.extract_input_identifiers(child);
                    return;
                }
            }
        }
    }

    /// Extract identifiers from a `function_arguments` node.
    fn extract_input_identifiers(&mut self, args_node: tree_sitter::Node<'a>) {
        let count = args_node.child_count();
        for i in 0..count {
            if let Some(child) = args_node.child(i) {
                if child.kind() == "identifier" {
                    let pos = child.start_position();
                    self.add_def(VarDef {
                        name: self.node_text(child).to_string(),
                        kind: DefKind::InputArg,
                        byte_range: child.start_byte()..child.end_byte(),
                        effect_start: child.start_byte(),
                        partial: false,
                        multi_output: false,
                        prealloc: false,
                        block_start: 0,
                        line: pos.row + 1,
                        column: pos.column + 1,
                    });
                }
                // Skip `ignored_argument` (~).
            }
        }
    }

    // -- lambda -------------------------------------------------------------

    /// Visit a `lambda` node (`@(x, y) expr`).
    ///
    /// Creates a Lambda scope, extracts parameters, then visits the body.
    fn visit_lambda(&mut self, node: tree_sitter::Node<'a>) {
        let pos = node.start_position();

        self.push_scope(Scope {
            name: String::new(),
            kind: ScopeKind::Lambda,
            byte_range: node.start_byte()..node.end_byte(),
            line: pos.row + 1,
            defs: Vec::new(),
            uses: Vec::new(),
            children: Vec::new(),
            parent: None, // set by push_scope
        });

        // Extract lambda parameters. In tree-sitter-matlab, the lambda's
        // arguments node contains the parameter identifiers.
        let count = node.child_count();
        for i in 0..count {
            if let Some(child) = node.child(i) {
                if child.kind() == "arguments" || child.kind() == "lambda_arguments" {
                    let arg_count = child.child_count();
                    for j in 0..arg_count {
                        if let Some(arg) = child.child(j) {
                            if arg.kind() == "identifier" {
                                let arg_pos = arg.start_position();
                                self.add_def(VarDef {
                                    name: self.node_text(arg).to_string(),
                                    kind: DefKind::LambdaParam,
                                    byte_range: arg.start_byte()..arg.end_byte(),
                                    effect_start: arg.start_byte(),
                                    partial: false,
                                    multi_output: false,
                                    prealloc: false,
                                    block_start: 0,
                                    line: arg_pos.row + 1,
                                    column: arg_pos.column + 1,
                                });
                            }
                        }
                    }
                }
            }
        }

        // Visit body (all children that are not the arguments).
        for i in 0..count {
            if let Some(child) = node.child(i) {
                let kind = child.kind();
                if kind == "arguments" || kind == "lambda_arguments" {
                    continue;
                }
                // Skip punctuation tokens like `@`, `(`, `)`.
                if kind == "@" || kind == "(" || kind == ")" || kind == "," {
                    continue;
                }
                self.visit_node(child);
            }
        }

        // A variable the lambda reads but does not bind is captured from the
        // enclosing scope: a use there, or the assignment looks dead.
        if let Some(lambda) = self.current_scope() {
            let captured: Vec<VarUse> = self.scopes[lambda]
                .uses
                .iter()
                .filter(|u| !self.scopes[lambda].defs.iter().any(|d| d.name == u.name))
                .cloned()
                .collect();
            self.pop_scope();
            if let Some(parent) = self.current_scope() {
                self.scopes[parent].uses.extend(captured);
            }
        } else {
            self.pop_scope();
        }
    }

    // -- assignment ---------------------------------------------------------

    /// Visit an `assignment` node.
    ///
    /// Extracts LHS identifiers as definitions, then visits the RHS as usages.
    fn visit_assignment(&mut self, node: tree_sitter::Node<'a>) {
        // Process LHS: definitions.
        if let Some(lhs) = node.child_by_field_name("left") {
            let before = self
                .current_scope()
                .map_or(0, |i| self.scopes[i].defs.len());
            self.extract_assignment_lhs(lhs, node.end_byte());
            let prealloc = node
                .child_by_field_name("right")
                .is_some_and(|r| self.is_prealloc_call(r));
            let block_start = node.parent().map_or(0, |p| p.start_byte());
            if let Some(idx) = self.current_scope() {
                for def in self.scopes[idx].defs.iter_mut().skip(before) {
                    def.block_start = block_start;
                    def.prealloc = prealloc && lhs.kind() == "identifier";
                }
            }
        }

        // Process RHS: usages (just recurse normally).
        if let Some(rhs) = node.child_by_field_name("right") {
            self.visit_node(rhs);
        }
    }

    /// Whether `node` is a call that creates a preallocated array.
    fn is_prealloc_call(&self, node: tree_sitter::Node<'a>) -> bool {
        const PREALLOC: &[&str] = &[
            "zeros", "ones", "cell", "nan", "NaN", "inf", "Inf", "false", "true", "eye", "rand",
        ];
        node.kind() == "function_call"
            && node
                .child_by_field_name("name")
                .is_some_and(|n| PREALLOC.contains(&self.node_text(n)))
    }

    /// Extract definitions from an assignment LHS.
    ///
    /// `effect` is where the assignment takes effect: the end of the statement.
    fn extract_assignment_lhs(&mut self, lhs: tree_sitter::Node<'a>, effect: usize) {
        match lhs.kind() {
            "identifier" => {
                let pos = lhs.start_position();
                self.add_def(VarDef {
                    name: self.node_text(lhs).to_string(),
                    kind: DefKind::Assignment,
                    byte_range: lhs.start_byte()..lhs.end_byte(),
                    effect_start: effect,
                    partial: false,
                    multi_output: false,
                    prealloc: false,
                    block_start: 0,
                    line: pos.row + 1,
                    column: pos.column + 1,
                });
            }
            "multioutput_variable" => {
                let count = lhs.child_count();
                for i in 0..count {
                    if let Some(child) = lhs.child(i) {
                        if child.kind() == "identifier" {
                            let pos = child.start_position();
                            self.add_def(VarDef {
                                name: self.node_text(child).to_string(),
                                kind: DefKind::Assignment,
                                byte_range: child.start_byte()..child.end_byte(),
                                effect_start: effect,
                                partial: false,
                                multi_output: true,
                                prealloc: false,
                                block_start: 0,
                                line: pos.row + 1,
                                column: pos.column + 1,
                            });
                        }
                        // Skip `ignored_argument` (~) and punctuation.
                    }
                }
            }
            // `A(i) = v`, `s.f = v`, `C{i} = v`: a partial assignment.
            "function_call" | "field_expression" | "cell_index" => {
                self.partial_assignment(lhs, effect);
            }
            _ => {
                // Unknown LHS shape — recurse for safety.
                self.visit_node(lhs);
            }
        }
    }

    /// A partial assignment (`A(i) = v`, `s.f.g = v`, `C{i} = v`) defines its
    /// root variable, creating it when it does not exist yet. It also reads the
    /// previous value, which is recorded as a use only when there is one:
    /// the first `A(2) = 5` has nothing to read.
    fn partial_assignment(&mut self, lhs: tree_sitter::Node<'a>, effect: usize) {
        let Some(root) = Self::root_identifier(lhs) else {
            self.visit_node(lhs);
            return;
        };
        let name = self.node_text(root).to_string();
        let defined_before = self
            .current_scope()
            .is_some_and(|idx| self.scopes[idx].defs.iter().any(|d| d.name == name));

        // Index expressions and the like are plain uses.
        self.visit_node(lhs);
        if !defined_before {
            if let Some(idx) = self.current_scope() {
                let at = root.start_byte();
                self.scopes[idx]
                    .uses
                    .retain(|u| !(u.name == name && u.byte_range.start == at));
            }
        }
        let pos = root.start_position();
        self.add_def(VarDef {
            name,
            kind: DefKind::Assignment,
            byte_range: root.start_byte()..root.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            effect_start: effect,
            partial: true,
            multi_output: false,
            prealloc: false,
            block_start: 0,
        });
    }

    /// The variable an indexed or field expression is rooted at (`x` in
    /// `x.y(3).z`).
    fn root_identifier(mut node: tree_sitter::Node<'a>) -> Option<tree_sitter::Node<'a>> {
        loop {
            node = match node.kind() {
                "identifier" => return Some(node),
                "function_call" => node.child_by_field_name("name")?,
                "field_expression" => node.child_by_field_name("object")?,
                "cell_index" => node
                    .child_by_field_name("name")
                    .or_else(|| node.named_child(0))?,
                _ => return None,
            };
        }
    }

    // -- for_statement ------------------------------------------------------

    /// Visit a `for_statement` node.
    ///
    /// Extracts the iterator variable as a definition, then visits the body.
    fn visit_for_statement(&mut self, node: tree_sitter::Node<'a>) {
        // The for statement has an `iterator` child that contains the loop
        // variable and the range expression.
        let count = node.child_count();
        for i in 0..count {
            if let Some(child) = node.child(i) {
                if child.kind() == "iterator" {
                    self.visit_iterator(child);
                } else {
                    self.visit_node(child);
                }
            }
        }
    }

    /// Visit an `iterator` node inside a for-statement.
    ///
    /// The first identifier child is the loop variable (definition).
    /// Everything else (the range expression) is visited for usages.
    fn visit_iterator(&mut self, node: tree_sitter::Node<'a>) {
        let mut found_var = false;
        let count = node.child_count();
        for i in 0..count {
            if let Some(child) = node.child(i) {
                if !found_var && child.kind() == "identifier" {
                    // This is the loop variable.
                    let pos = child.start_position();
                    self.add_def(VarDef {
                        name: self.node_text(child).to_string(),
                        kind: DefKind::ForIterator,
                        byte_range: child.start_byte()..child.end_byte(),
                        effect_start: child.start_byte(),
                        partial: false,
                        multi_output: false,
                        prealloc: false,
                        block_start: 0,
                        line: pos.row + 1,
                        column: pos.column + 1,
                    });
                    found_var = true;
                } else if child.kind() == "=" {
                    // Skip the `=` token.
                    continue;
                } else {
                    // Range expression — visit for usages.
                    self.visit_node(child);
                }
            }
        }
    }

    // -- global / persistent ------------------------------------------------

    /// Visit a `global_operator` or `persistent_operator` node.
    ///
    /// Each child identifier is a declaration.
    fn visit_global_persistent(&mut self, node: tree_sitter::Node<'a>, kind: DefKind) {
        let count = node.child_count();
        for i in 0..count {
            if let Some(child) = node.child(i) {
                if child.kind() == "identifier" {
                    let pos = child.start_position();
                    self.add_def(VarDef {
                        name: self.node_text(child).to_string(),
                        kind,
                        byte_range: child.start_byte()..child.end_byte(),
                        effect_start: child.start_byte(),
                        partial: false,
                        multi_output: false,
                        prealloc: false,
                        block_start: 0,
                        line: pos.row + 1,
                        column: pos.column + 1,
                    });
                }
            }
        }
    }

    // -- identifier (the tricky one) ----------------------------------------

    /// Visit a bare `identifier` node.
    ///
    /// Determines from parent context whether this identifier is a definition
    /// (already captured elsewhere), a function/field name (skip), or a
    /// genuine variable usage.
    fn visit_identifier(&mut self, node: tree_sitter::Node<'a>) {
        let Some(parent) = node.parent() else {
            // Orphan identifier — treat as usage to be safe.
            self.record_use(node);
            return;
        };

        match parent.kind() {
            // Already handled by visit_assignment.
            "assignment" => {
                if parent
                    .child_by_field_name("left")
                    .map(|l| l.id() == node.id())
                    .unwrap_or(false)
                {
                    // This is the simple LHS identifier — already captured.
                    return;
                }
                // Otherwise it's on the RHS — it's a usage.
                self.record_use(node);
            }

            // Already handled by extract_assignment_lhs.
            "multioutput_variable" => {
                // Check if the multioutput_variable is the LHS of an assignment.
                if let Some(grandparent) = parent.parent() {
                    if grandparent.kind() == "assignment"
                        && grandparent
                            .child_by_field_name("left")
                            .map(|l| l.id() == parent.id())
                            .unwrap_or(false)
                    {
                        // Already captured as assignment def.
                        return;
                    }
                    // multioutput_variable in function_output context.
                    if grandparent.kind() == "function_output" {
                        return;
                    }
                }
                self.record_use(node);
            }

            // Function definition name — skip (not a variable usage).
            "function_definition" => {
                if parent
                    .child_by_field_name("name")
                    .map(|n| n.id() == node.id())
                    .unwrap_or(false)
                {
                    return;
                }
                // Could be a child that's not the name — e.g., part of
                // the body that was somehow an identifier child. Visit as use.
                self.record_use(node);
            }

            // function_arguments / function_output — already handled.
            "function_arguments" | "function_output" => {
                // Already captured as InputArg / OutputArg.
            }

            // global/persistent — already handled.
            "global_operator" | "persistent_operator" => {
                // Already captured as Global / Persistent.
            }

            // For-loop iterator — already handled.
            "iterator" => {
                // Already captured as ForIterator. But the range expression
                // identifiers also have parent `iterator`. We need to check if
                // this is the first identifier (the loop var) or a later one.
                // The first identifier child of `iterator` is the loop var.
                let first_ident = Self::first_identifier_child(parent);
                if first_ident.map(|n| n.id() == node.id()).unwrap_or(false) {
                    // This is the loop variable — already captured.
                    return;
                }
                // Otherwise it's part of the range expression.
                self.record_use(node);
            }

            // Lambda arguments — already handled.
            "arguments" | "lambda_arguments" => {
                // Only skip if the parent's parent is a lambda.
                if let Some(grandparent) = parent.parent() {
                    if grandparent.kind() == "lambda" {
                        return;
                    }
                }
                self.record_use(node);
            }

            // Function call name: count as usage (could be array indexing).
            "function_call" => {
                // The callee is a use because it could be indexing (`A(i)`),
                // but one that checks can tell from a plain variable use.
                let is_callee = parent
                    .child_by_field_name("name")
                    .is_some_and(|n| n.id() == node.id());
                self.record_use_as(node, is_callee);
            }

            // Field expression: only the `object` part is a usage, not the
            // `field` part.
            "field_expression" => {
                if is_field_name(node) {
                    // This is the field name (e.g., `field` in `obj.field`).
                    // Not a standalone variable usage.
                    return;
                }
                // This is the object part — it's a usage.
                self.record_use(node);
            }

            // Class definition name — skip.
            "class_definition" => {
                if parent
                    .child_by_field_name("name")
                    .map(|n| n.id() == node.id())
                    .unwrap_or(false)
                {
                    return;
                }
                self.record_use(node);
            }

            // Default: it's a variable usage.
            _ => {
                self.record_use(node);
            }
        }
    }

    /// Find the first `identifier` child of a node.
    fn first_identifier_child(node: tree_sitter::Node) -> Option<tree_sitter::Node> {
        let count = node.child_count();
        for i in 0..count {
            if let Some(child) = node.child(i) {
                if child.kind() == "identifier" {
                    return Some(child);
                }
            }
        }
        None
    }

    /// Record a variable usage at the given identifier node.
    fn record_use(&mut self, node: tree_sitter::Node) {
        self.record_use_as(node, false);
    }

    /// Record a use; `is_call_target` marks the callee of `name(...)`.
    fn record_use_as(&mut self, node: tree_sitter::Node, is_call_target: bool) {
        let pos = node.start_position();
        self.add_use(VarUse {
            name: self.node_text(node).to_string(),
            byte_range: node.start_byte()..node.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            is_call_target,
        });
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Parser;

    /// Create a parser and parse MATLAB source, returning the tree.
    fn parse(source: &str) -> Tree {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_matlab::LANGUAGE.into())
            .expect("failed to load tree-sitter-matlab");
        parser.parse(source, None).expect("parse failed")
    }

    // -- basic function file ------------------------------------------------

    #[test]
    fn single_function_with_args() {
        let source = "function y = foo(x)\n    y = x + 1;\nend\n";
        let tree = parse(source);
        let table = SymbolTable::build(&tree, source);

        // Should have exactly one scope: the main function.
        assert_eq!(table.scopes.len(), 1);
        let scope = table.root_scope();
        assert_eq!(scope.kind, ScopeKind::Function);
        assert_eq!(scope.name, "foo");

        // Definitions: y (OutputArg), x (InputArg), y (Assignment).
        assert!(scope.is_defined("y"));
        assert!(scope.is_defined("x"));

        let x_defs = scope.defs_of("x");
        assert_eq!(x_defs.len(), 1);
        assert_eq!(x_defs[0].kind, DefKind::InputArg);

        let y_defs = scope.defs_of("y");
        assert_eq!(y_defs.len(), 2); // OutputArg + Assignment
        assert!(y_defs.iter().any(|d| d.kind == DefKind::OutputArg));
        assert!(y_defs.iter().any(|d| d.kind == DefKind::Assignment));

        // Usages: x (in `x + 1`).
        assert!(scope.is_used("x"));
    }

    // -- script file --------------------------------------------------------

    #[test]
    fn script_file() {
        let source = "x = 1;\ny = x + 2;\ndisp(y);\n";
        let tree = parse(source);
        let table = SymbolTable::build(&tree, source);

        assert_eq!(table.scopes.len(), 1);
        let scope = table.root_scope();
        assert_eq!(scope.kind, ScopeKind::Script);

        // x and y are assigned.
        assert!(scope.is_defined("x"));
        assert!(scope.is_defined("y"));

        // x is used in `x + 2`, y is used in `disp(y)`.
        assert!(scope.is_used("x"));
        assert!(scope.is_used("y"));

        // disp is used (function_call name counts as usage).
        assert!(scope.is_used("disp"));
    }

    // -- local functions ----------------------------------------------------

    #[test]
    fn main_and_local_functions() {
        let source = "\
function y = main(x)
    y = helper(x);
end

function z = helper(a)
    z = a * 2;
end
";
        let tree = parse(source);
        let table = SymbolTable::build(&tree, source);

        // Two scopes: main (Function) + helper (LocalFunction).
        assert_eq!(table.scopes.len(), 2);

        let main_scope = &table.scopes[0];
        assert_eq!(main_scope.kind, ScopeKind::Function);
        assert_eq!(main_scope.name, "main");

        let helper_scope = &table.scopes[1];
        assert_eq!(helper_scope.kind, ScopeKind::LocalFunction);
        assert_eq!(helper_scope.name, "helper");

        // main defines y (output), x (input), y (assignment).
        assert!(main_scope.is_defined("y"));
        assert!(main_scope.is_defined("x"));

        // helper defines z (output), a (input), z (assignment).
        assert!(helper_scope.is_defined("z"));
        assert!(helper_scope.is_defined("a"));
    }

    // -- for loop -----------------------------------------------------------

    #[test]
    fn for_loop_iterator() {
        let source = "\
function foo()
    for i = 1:10
        disp(i);
    end
end
";
        let tree = parse(source);
        let table = SymbolTable::build(&tree, source);

        let scope = table.root_scope();
        assert!(scope.is_defined("i"));

        let i_defs = scope.defs_of("i");
        assert_eq!(i_defs.len(), 1);
        assert_eq!(i_defs[0].kind, DefKind::ForIterator);

        // i is used in disp(i).
        assert!(scope.is_used("i"));
    }

    // -- global / persistent ------------------------------------------------

    #[test]
    fn global_and_persistent() {
        let source = "\
function foo()
    global gVar;
    persistent pVar;
    x = gVar + pVar;
end
";
        let tree = parse(source);
        let table = SymbolTable::build(&tree, source);

        let scope = table.root_scope();

        let g_defs = scope.defs_of("gVar");
        assert_eq!(g_defs.len(), 1);
        assert_eq!(g_defs[0].kind, DefKind::Global);

        let p_defs = scope.defs_of("pVar");
        assert_eq!(p_defs.len(), 1);
        assert_eq!(p_defs[0].kind, DefKind::Persistent);

        // gVar and pVar are used on the RHS.
        assert!(scope.is_used("gVar"));
        assert!(scope.is_used("pVar"));
    }

    // -- multioutput assignment ---------------------------------------------

    #[test]
    fn multioutput_assignment() {
        let source = "\
function foo()
    [a, b, ~] = deal(1, 2, 3);
end
";
        let tree = parse(source);
        let table = SymbolTable::build(&tree, source);

        let scope = table.root_scope();
        assert!(scope.is_defined("a"));
        assert!(scope.is_defined("b"));
        // `~` (ignored_argument) should NOT appear as a def.
        assert!(!scope.is_defined("~"));
    }

    // -- field expressions --------------------------------------------------

    #[test]
    fn field_expression_object_is_usage() {
        let source = "\
function foo(s)
    x = s.field;
end
";
        let tree = parse(source);
        let table = SymbolTable::build(&tree, source);

        let scope = table.root_scope();
        // `s` is used (the object part of s.field).
        assert!(scope.is_used("s"));
        // `field` is NOT a variable usage.
        assert!(!scope.is_used("field"));
    }

    // -- lambda -------------------------------------------------------------

    #[test]
    fn lambda_creates_scope() {
        let source = "\
function foo()
    f = @(x, y) x + y;
end
";
        let tree = parse(source);
        let table = SymbolTable::build(&tree, source);

        // Two scopes: foo (Function) + the lambda.
        assert_eq!(table.scopes.len(), 2);

        let lambda_scope = &table.scopes[1];
        assert_eq!(lambda_scope.kind, ScopeKind::Lambda);
        assert!(lambda_scope.is_defined("x"));
        assert!(lambda_scope.is_defined("y"));
        assert!(lambda_scope.is_used("x"));
        assert!(lambda_scope.is_used("y"));

        let x_defs = lambda_scope.defs_of("x");
        assert_eq!(x_defs.len(), 1);
        assert_eq!(x_defs[0].kind, DefKind::LambdaParam);
    }

    // -- helper methods -----------------------------------------------------

    #[test]
    fn defined_names_and_used_names() {
        let source = "\
function y = foo(x)
    y = x + 1;
end
";
        let tree = parse(source);
        let table = SymbolTable::build(&tree, source);

        let scope = table.root_scope();
        let defs = scope.defined_names();
        assert!(defs.contains(&"x"));
        assert!(defs.contains(&"y"));

        let uses = scope.used_names();
        assert!(uses.contains(&"x"));
    }

    // -- scope_at -----------------------------------------------------------

    #[test]
    fn scope_at_finds_innermost() {
        let source = "\
function foo()
    f = @(x) x + 1;
end
";
        let tree = parse(source);
        let table = SymbolTable::build(&tree, source);

        // The lambda scope is nested inside foo.
        assert_eq!(table.scopes.len(), 2);

        let lambda = &table.scopes[1];
        let mid = (lambda.byte_range.start + lambda.byte_range.end) / 2;
        let found = table.scope_at(mid).unwrap();
        assert_eq!(found.kind, ScopeKind::Lambda);
    }

    // -- function_scopes helper ---------------------------------------------

    #[test]
    fn function_scopes_excludes_lambdas() {
        let source = "\
function foo()
    f = @(x) x;
end

function bar()
end
";
        let tree = parse(source);
        let table = SymbolTable::build(&tree, source);

        let func_scopes = table.function_scopes();
        // foo (Function) + bar (LocalFunction), NOT the lambda.
        assert_eq!(func_scopes.len(), 2);
        assert!(func_scopes.iter().all(|s| s.kind != ScopeKind::Lambda));
    }

    // -- nested function ----------------------------------------------------

    #[test]
    fn nested_function_scope() {
        let source = "\
function outer()
    x = inner();
    function y = inner()
        y = 42;
    end
end
";
        let tree = parse(source);
        let table = SymbolTable::build(&tree, source);

        // Two scopes: outer + inner (NestedFunction).
        assert_eq!(table.scopes.len(), 2);

        let outer = &table.scopes[0];
        assert_eq!(outer.kind, ScopeKind::Function);

        let inner = &table.scopes[1];
        assert_eq!(inner.kind, ScopeKind::NestedFunction);
        assert_eq!(inner.name, "inner");

        // outer should have `inner` as a NestedFunction def.
        let inner_defs = outer.defs_of("inner");
        assert_eq!(inner_defs.len(), 1);
        assert_eq!(inner_defs[0].kind, DefKind::NestedFunction);
    }

    // -- function_call name is a usage --------------------------------------

    #[test]
    fn function_call_name_is_usage() {
        let source = "\
function foo(A)
    x = A(1);
end
";
        let tree = parse(source);
        let table = SymbolTable::build(&tree, source);

        let scope = table.root_scope();
        // `A` is used (as a function_call name — could be array indexing).
        assert!(scope.is_used("A"));
    }

    // -- parent/child wiring ------------------------------------------------

    #[test]
    fn parent_child_wiring() {
        let source = "\
function foo()
    f = @(x) x;
end
";
        let tree = parse(source);
        let table = SymbolTable::build(&tree, source);

        let foo = &table.scopes[0];
        let lambda = &table.scopes[1];

        assert_eq!(foo.children, vec![1]);
        assert_eq!(lambda.parent, Some(0));
    }
}
