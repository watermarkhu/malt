//! # MP_STATEMENTS_ENGINE: MonkeyProof statements and expressions
//!
//! ```malt
//! id = "MP_STATEMENTS_ENGINE"
//! title = "MonkeyProof statements and expressions"
//! category = "monkeyproof"
//! severity = "info"
//! fix = false
//! slug = "monkeyproof-statements"
//! icon = "lucide/code"
//! note = "Not part of MATLAB's Code Analyzer, and malt is not affiliated with MonkeyProof Solutions. Off unless the monkeyproof preset or category is on."
//! ```
//!
//! ## Rule
//!
//! The checks of the statements and expressions chapter of the
//! [MonkeyProof Coding Standard for MATLAB](https://doc.monkeyproofsolutions.nl/code-checker-for-matlab/monkeyproof-coding-standard-for-matlab/statements/statements.html) that can be checked automatically.
//!
//! The checks are off by default. Switch them on with `preset = "monkeyproof"`, or with
//! `[lint.categories] monkeyproof = "warn"`, or one at a time under `[lint.rules]`.
//! Every check works on one file's syntax tree, so it reports what the file itself
//! shows and stays silent when it would have to guess. Where a check cannot see
//! everything, the limit is given here.
//!
//! `MP_STAT_17` (all called functions are known to MATLAB) is not implemented: it needs
//! MATLAB's own function list, toolboxes and the user's path, which a single file does
//! not provide.
//!
//! ## Check IDs
//!
//! | Check ID | Severity | Fix | Description |
//! | -------- | -------- | --- | ----------- |
//! | MP_STAT_1 | info | no | A constant (`MAX_SIZE = 100`, capitals, literal value) is defined after other code in a function body. Only the top level of the body is looked at. |
//! | MP_STAT_2 | warning | no | A number literal in an expression that is not in the allowed list. |
//! | MP_STAT_3 | info | no | A well-known function, or a function of the same file, is used without parentheses (`x = rand * 2`, `tic`). |
//! | MP_STAT_4 | info | no | More than one statement starts on a line. |
//! | MP_STAT_5 | info | no | An `if` has neither `elseif` nor `else`, and is not error or input handling. |
//! | MP_STAT_6 | warning | no | A `switch` has no `otherwise`. |
//! | MP_STAT_7 | warning | no | An arithmetic expression is an operand of a logical operator, or a logical expression an operand of an arithmetic one. |
//! | MP_STAT_8 | warning | no | Different logical operators are mixed without parentheses (`a && b \|\| c`). |
//! | MP_STAT_9 | warning | no | Different arithmetic operators are mixed without parentheses (`a * b + c`). |
//! | MP_STAT_10 | warning | no | An `assert` call lacks an error identifier and a message as its second and third inputs. |
//! | MP_STAT_11 | warning | no | A `global` declaration. |
//! | MP_STAT_12 | warning | no | A condition of `if`, `elseif` or `while` that is always true or always false. |
//! | MP_STAT_13 | info | no | A field is added to a struct after the struct was used. Only structs created by `struct(...)` in the same block are followed. |
//! | MP_STAT_18 | warning | no | A `try` has no `catch`, an empty `catch`, or a `catch` that never raises an error. |
//! | MP_STAT_19 | warning | no | `==` or `~=` with a fractional number, `pi` or the result of a floating-point function. |
//! | MP_STAT_21 | info | no | A number written without a zero before the decimal point (`.5`). |
//! | MP_STAT_22 | warning | no | The variable `ans` is read or written by name. |
//! | MP_STAT_23 | warning | no | A call of a debugger function such as `keyboard` or `dbstop`. |
//! | MP_STAT_24 | warning | no | An `if` with an `elseif` has no `else`. |
//! | MP_STAT_25 | error | no | The constructor of an abstract class defined in the same file is called. |
//! | MP_STAT_26 | info | no | A statement in command syntax with arguments (`hold on`). |
//! | MP_STAT_27 | warning | no | A function that reads or writes another workspace (`evalin`, `assignin`, `who`, `clc`, ...). |
//!
//! ## Configuration
//!
//! Each parameter belongs to the table of its check. A list parameter replaces the
//! default list when it is set.
//!
//! ```toml
//! [lint.rules.MP_STAT_2]
//! allowed = [0, 1, 2]        # values that are not magic, by absolute value
//! check_indices = false      # also report numbers inside x(5) on an assigned name
//!
//! [lint.rules.MP_STAT_3]
//! functions = ["rand", "randn", "tic", "toc", "clock", "now", "cputime", "date", "pwd",
//!              "computer", "version", "mfilename", "gcf", "gca", "figure", "clc", "clf",
//!              "drawnow", "beep", "pause"]
//! local_functions = true     # also functions defined in the same file
//!
//! [lint.rules.MP_STAT_5]
//! exit_functions = ["error", "throw", "rethrow", "throwAsCaller"]
//! input_checks = ["isempty", "isfield", "isa", "ischar", "isnumeric"]  # shortened here
//!
//! [lint.rules.MP_STAT_12]
//! allow_endless_loop = false # accept `while true`
//!
//! [lint.rules.MP_STAT_18]
//! error_functions = ["error", "rethrow", "throw", "throwAsCaller"]
//!
//! [lint.rules.MP_STAT_19]
//! functions = ["sqrt", "sin", "cos", "exp", "log", "mean"]  # shortened here
//!
//! [lint.rules.MP_STAT_23]
//! functions = ["keyboard", "dbstop", "dbclear", "dbcont", "dbstep", "dbquit", "dbup",
//!              "dbdown", "dbstack", "dbstatus", "dbtype"]
//!
//! [lint.rules.MP_STAT_26]
//! allow = ["hold", "format"] # command names to accept
//!
//! [lint.rules.MP_STAT_27]
//! functions = ["evalin", "assignin", "inputname", "who", "whos", "evalc", "clc"]
//! special_cases = true       # exist(x, "var"), load without output, clear / clear all
//! ```
//!
//! ## Notes on individual checks
//!
//! - `MP_STAT_3` trusts a name only when it is on the list or defined in the file, and
//!   never when the file also uses it as a variable. Constants such as `pi`, `eps`, `Inf`
//!   and `NaN` are values, not calls.
//! - `MP_STAT_5` leaves out an `if` whose body only errors or returns, and an `if` that
//!   tests `nargin`, `nargout`, `exist`, or a predicate on an input of the function. An
//!   `if` with an `elseif` is reported by `MP_STAT_24` only.
//! - `MP_STAT_7` judges only operands that are clearly arithmetic or clearly logical. A
//!   variable or a call may hold either type and is never reported.
//! - `MP_STAT_9` follows the standard literally: `a + b - c` mixes two operators and is
//!   reported, `a * b * c` is not.
//! - `MP_STAT_12` also reports `x > 0 || true`, where one side decides the result.
//! - `MP_STAT_13` counts any mention of the struct as a use, including reading a field.
//! - `MP_STAT_18` reads the rule strictly: a `catch` that only logs or assigns a default
//!   hides the error from the caller and is reported.
//! - `MP_STAT_25` knows only the abstract classes defined in the same file.
//!
//! ## Examples
//!
//! ### Incorrect
//!
//! ```matlab
//! function out = scale(in)
//! out = in * 1.8 + 32;          % MP_STAT_2, MP_STAT_9
//! if out == .5, out = 0; end    % MP_STAT_19, MP_STAT_21, MP_STAT_4, MP_STAT_5
//! hold on                       % MP_STAT_26
//! end
//! ```
//!
//! ### Correct
//!
//! ```matlab
//! function out = scale(in)
//! FACTOR = 1.8;
//! OFFSET = 32;
//! out = (in * FACTOR) + OFFSET;
//! if abs(out - 0.5) < 1e-9
//!     out = 0;
//! else
//!     % Keep the converted value.
//! end
//! hold("on")
//! end
//! ```

use crate::analysis::command::Command;
use malt_core::{Config, Rule};

use super::{Check, GroupEngine};

mod check_abstract_class;
mod check_ans;
mod check_assert_inputs;
mod check_command_syntax;
mod check_constant_condition;
mod check_constants_at_top;
mod check_debug_functions;
mod check_elseif_else;
mod check_empty_parentheses;
mod check_float_equality;
mod check_global;
mod check_if_else;
mod check_logical_parentheses;
mod check_magic_numbers;
mod check_math_parentheses;
mod check_mixed_operands;
mod check_one_statement;
mod check_struct_fields;
mod check_switch_otherwise;
mod check_try_catch;
mod check_workspace_access;
mod check_zero_before_point;

/// The checks of this group.
pub(crate) const CHECKS: &[Check] = &[
    Check {
        id: "MP_STAT_1",
        level: 9,
        run: check_constants_at_top::run,
    },
    Check {
        id: "MP_STAT_2",
        level: 5,
        run: check_magic_numbers::run,
    },
    Check {
        id: "MP_STAT_3",
        level: 9,
        run: check_empty_parentheses::run,
    },
    Check {
        id: "MP_STAT_4",
        level: 9,
        run: check_one_statement::run,
    },
    Check {
        id: "MP_STAT_5",
        level: 9,
        run: check_if_else::run,
    },
    Check {
        id: "MP_STAT_6",
        level: 4,
        run: check_switch_otherwise::run,
    },
    Check {
        id: "MP_STAT_7",
        level: 3,
        run: check_mixed_operands::run,
    },
    Check {
        id: "MP_STAT_8",
        level: 5,
        run: check_logical_parentheses::run,
    },
    Check {
        id: "MP_STAT_9",
        level: 5,
        run: check_math_parentheses::run,
    },
    Check {
        id: "MP_STAT_10",
        level: 5,
        run: check_assert_inputs::run,
    },
    Check {
        id: "MP_STAT_11",
        level: 4,
        run: check_global::run,
    },
    Check {
        id: "MP_STAT_12",
        level: 3,
        run: check_constant_condition::run,
    },
    Check {
        id: "MP_STAT_13",
        level: 9,
        run: check_struct_fields::run,
    },
    Check {
        id: "MP_STAT_18",
        level: 5,
        run: check_try_catch::run,
    },
    Check {
        id: "MP_STAT_19",
        level: 3,
        run: check_float_equality::run,
    },
    Check {
        id: "MP_STAT_21",
        level: 9,
        run: check_zero_before_point::run,
    },
    Check {
        id: "MP_STAT_22",
        level: 3,
        run: check_ans::run,
    },
    Check {
        id: "MP_STAT_23",
        level: 3,
        run: check_debug_functions::run,
    },
    Check {
        id: "MP_STAT_24",
        level: 4,
        run: check_elseif_else::run,
    },
    Check {
        id: "MP_STAT_25",
        level: 1,
        run: check_abstract_class::run,
    },
    Check {
        id: "MP_STAT_26",
        level: 7,
        run: check_command_syntax::run,
    },
    Check {
        id: "MP_STAT_27",
        level: 3,
        run: check_workspace_access::run,
    },
];

/// Factory constructor invoked by inventory.
pub fn from_config(config: &Config) -> Box<dyn Rule> {
    GroupEngine::boxed(
        "MP_STATEMENTS_ENGINE",
        "MonkeyProof statements and expressions",
        CHECKS,
        config,
    )
}

inventory::submit!(crate::RuleRegistration::new(
    "MP_STATEMENTS_ENGINE",
    from_config
));

// ---------------------------------------------------------------------------
// Helpers shared by the checks of this group.
// ---------------------------------------------------------------------------

use std::collections::HashSet;

use tree_sitter::Node;

/// The source text of `node`.
pub(crate) fn text<'s>(node: Node, source: &'s str) -> &'s str {
    source.get(node.byte_range()).unwrap_or("")
}

/// Visit every node under `root` in pre-order, with its parent (`None` for `root`).
/// Iterative, so deep trees cannot overflow the stack, and linear in the tree size.
pub(crate) fn walk<'t>(root: Node<'t>, mut visit: impl FnMut(Node<'t>, Option<Node<'t>>)) {
    let mut cursor = root.walk();
    let mut stack: Vec<Node<'t>> = Vec::new();
    loop {
        let node = cursor.node();
        visit(node, stack.last().copied());
        if cursor.goto_first_child() {
            stack.push(node);
            continue;
        }
        loop {
            if cursor.goto_next_sibling() {
                break;
            }
            if !cursor.goto_parent() {
                return;
            }
            stack.pop();
        }
    }
}

/// `node` with any wrapping parentheses removed.
pub(crate) fn peel(mut node: Node) -> Node {
    while node.kind() == "parenthesis" {
        match node.named_child(0) {
            Some(inner) => node = inner,
            None => break,
        }
    }
    node
}

/// The operator token of a binary, boolean, comparison or unary operator node.
pub(crate) fn operator<'s>(node: Node, source: &'s str) -> Option<&'s str> {
    let mut cursor = node.walk();
    let found = node
        .children(&mut cursor)
        .find(|child| !child.is_named() && !child.is_extra())
        .map(|child| text(child, source));
    found
}

/// The named, non-comment children of `node`.
pub(crate) fn statements_of(node: Node) -> Vec<Node> {
    let mut cursor = node.walk();
    let found = node
        .named_children(&mut cursor)
        .filter(|child| !child.is_extra() && child.kind() != "comment")
        .collect();
    found
}

/// The arguments of a `function_call`, in order.
pub(crate) fn call_arguments(call: Node) -> Vec<Node> {
    let mut cursor = call.walk();
    let args = call
        .named_children(&mut cursor)
        .find(|child| child.kind() == "arguments");
    match args {
        Some(args) => statements_of(args),
        None => Vec::new(),
    }
}

/// The identifier a `function_call` calls (`sqrt` in `sqrt(2)`); `None` for a dotted or
/// chained callee.
pub(crate) fn call_identifier<'s>(call: Node, source: &'s str) -> Option<&'s str> {
    let name = call.child_by_field_name("name")?;
    (call.kind() == "function_call" && name.kind() == "identifier").then(|| text(name, source))
}

/// The inner text of a string literal node (`'abc'` and `"abc"` give `abc`).
pub(crate) fn string_value<'s>(node: Node, source: &'s str) -> Option<&'s str> {
    if node.kind() != "string" {
        return None;
    }
    let raw = text(node, source);
    let inner = raw.get(1..raw.len().checked_sub(1)?)?;
    Some(inner)
}

/// The leftmost identifier of an assignment target: `x` in `x`, `x(3)`, `x.a(2).b`.
pub(crate) fn target_base(mut node: Node) -> Option<Node> {
    loop {
        match node.kind() {
            "identifier" => return Some(node),
            "function_call" => node = node.child_by_field_name("name")?,
            "field_expression" => node = node.child_by_field_name("object")?,
            _ => return None,
        }
    }
}

/// Every name the file binds as a variable: assignment targets, loop variables,
/// function inputs and outputs, `global` and `persistent` names, the `catch` identifier.
/// A name in this set is not trusted to be a function call.
pub(crate) fn variables<'s>(root: Node, source: &'s str) -> HashSet<&'s str> {
    let mut names = HashSet::new();
    walk(root, |node, parent| {
        let Some(parent) = parent else { return };
        match (parent.kind(), node.kind()) {
            ("assignment", _) => {
                if parent.child_by_field_name("left") == Some(node) {
                    if let Some(base) = target_base(node) {
                        names.insert(text(base, source));
                    }
                }
            }
            (
                "multioutput_variable"
                | "function_output"
                | "function_arguments"
                | "global_operator"
                | "persistent_operator"
                | "catch_clause"
                | "iterator",
                "identifier",
            ) => {
                if parent.kind() != "iterator" || parent.named_child(0) == Some(node) {
                    names.insert(text(node, source));
                }
            }
            ("arguments", "identifier") => {
                // The inputs of an anonymous function.
                if parent
                    .parent()
                    .is_some_and(|grand| grand.kind() == "lambda")
                {
                    names.insert(text(node, source));
                }
            }
            _ => {}
        }
        if parent.kind() == "multioutput_variable" && node.is_named() {
            if let Some(base) = target_base(node) {
                names.insert(text(base, source));
            }
        }
    });
    names
}

/// How a name is used as a callee.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum UseKind {
    /// `name(args)`.
    Call,
    /// `name` alone as a statement.
    Bare,
    /// `name arg arg` command syntax.
    Command,
}

/// A use of a plain name as a callee, found by [`callee_uses`].
pub(crate) struct CalleeUse<'a> {
    /// The name.
    pub name: &'a str,
    /// The way it is used.
    pub kind: UseKind,
    /// The name's own node (`command_name` for a command).
    pub name_node: Node<'a>,
    /// The whole call, bare identifier or command.
    pub node: Node<'a>,
    /// The argument nodes of a call. Empty for the other kinds.
    pub args: Vec<Node<'a>>,
    /// The argument words of a command. Empty for the other kinds.
    pub words: Vec<&'a str>,
    /// Whether the use is a statement of its own (the value is not used).
    pub statement: bool,
}

/// Every call, bare statement and command of a plain name under `root`.
pub(crate) fn callee_uses<'a>(root: Node<'a>, source: &'a str) -> Vec<CalleeUse<'a>> {
    let mut uses = Vec::new();
    walk(root, |node, parent| {
        let statement = parent.is_some_and(|p| matches!(p.kind(), "block" | "source_file"));
        match node.kind() {
            "function_call" => {
                let member = parent.is_some_and(|p| is_member(node, p));
                if let (false, Some(name)) = (member, call_identifier(node, source)) {
                    uses.push(CalleeUse {
                        name,
                        kind: UseKind::Call,
                        name_node: node.child_by_field_name("name").unwrap_or(node),
                        node,
                        args: call_arguments(node),
                        words: Vec::new(),
                        statement,
                    });
                }
            }
            "identifier" if statement => uses.push(CalleeUse {
                name: text(node, source),
                kind: UseKind::Bare,
                name_node: node,
                node,
                args: Vec::new(),
                words: Vec::new(),
                statement,
            }),
            "command" => {
                if let Some(command) = Command::from_node(node, source) {
                    if command.is_command() && !command.is_shell_escape() {
                        let bare = command.argument_nodes.is_empty();
                        uses.push(CalleeUse {
                            name: command.name,
                            kind: if bare {
                                UseKind::Bare
                            } else {
                                UseKind::Command
                            },
                            name_node: node.child(0).unwrap_or(node),
                            node,
                            args: Vec::new(),
                            words: command.arguments.clone(),
                            statement,
                        });
                    }
                }
            }
            _ => {}
        }
    });
    uses
}

/// Whether `node` is the member of a dotted access (`run` in `obj.run()`), not a plain name.
pub(crate) fn is_member(node: Node, parent: Node) -> bool {
    parent.kind() == "field_expression" && parent.child_by_field_name("object") != Some(node)
}

/// A list parameter that replaces `default` when the user sets it.
pub(crate) fn name_list(configured: Option<Vec<String>>, default: &[&str]) -> HashSet<String> {
    match configured {
        Some(list) => list.into_iter().collect(),
        None => default.iter().map(|name| (*name).to_string()).collect(),
    }
}

/// Run `check` over `source` with the check's config, for tests.
#[cfg(test)]
pub(crate) fn lint(source: &str, toml: &str) -> Vec<malt_core::Diagnostic> {
    super::lint(from_config, source, toml)
}

/// The rule IDs and (line, column) of the findings of one check, for tests.
#[cfg(test)]
pub(crate) fn hits(id: &str, source: &str, toml: &str) -> Vec<(usize, usize)> {
    lint(source, toml)
        .iter()
        .filter(|d| d.rule_id == id)
        .map(|d| (d.line, d.column))
        .collect()
}
