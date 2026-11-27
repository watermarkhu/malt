//! # MP_SECURITY_ENGINE: MonkeyProof security rules
//!
//! ```malt
//! id = "MP_SECURITY_ENGINE"
//! title = "MonkeyProof security rules"
//! category = "monkeyproof"
//! severity = "info"
//! fix = false
//! slug = "monkeyproof-security"
//! icon = "lucide/shield"
//! note = "Not part of MATLAB's Code Analyzer. Off unless the monkeyproof preset or category is on."
//! ```
//!
//! ## Rule
//!
//! The checks of the security chapter of the
//! [MonkeyProof Coding Standard for MATLAB](https://doc.monkeyproofsolutions.nl/code-checker-for-matlab/monkeyproof-coding-standard-for-matlab/security/security.html) that can be checked automatically.
//!
//! ## Check IDs
//!
//! | Check ID | Severity | Fix | Description |
//! | -------- | -------- | --- | ----------- |
//! | MP_SECURITY_1 | warning | no | Do not use `eval`; avoid `evalin`, `evalc` and `feval` with a literal name. |
//! | MP_SECURITY_2 | warning | no | Do not use the `!` shell escape. |
//! | MP_SECURITY_3 | warning | no | Do not set an empty `CertificateFilename` on `weboptions` or `HTTPOptions`. |
//! | MP_SECURITY_4 | warning | no | Set `AllowDoctype` to false when reading XML. |
//! | MP_SECURITY_5 | warning | no | Do not give variables such as `username` and `password` a constant value. |
//! | MP_SECURITY_6 | warning | no | Do not hard-code values for `Username` and `Password`. |
//!
//! Switch the group on with `[lint.categories] monkeyproof = "warn"` or `preset = "monkeyproof"`.
//! Every check can be switched on its own and given a severity under `[lint.rules.MP_SECURITY_n]`.
//!
//! ## Parameters
//!
//! | Check | Parameter | Default | Meaning |
//! | ----- | --------- | ------- | ------- |
//! | MP_SECURITY_1 | `functions` | `eval`, `evalin`, `evalc` | Functions flagged on every use. |
//! | MP_SECURITY_1 | `literal_only` | `feval` | Functions flagged only when the first argument is a string literal. |
//! | MP_SECURITY_5 | `names` | `username`, `loginname`, `password`, `passwd`, `credentials` | Variable names, compared ignoring case and underscores. |
//! | MP_SECURITY_6 | `functions` | `weboptions`, `webread`, `webwrite`, `websave`, `matlab.net.http.Credentials`, `Credentials` | Functions whose name-value pairs can hold credentials. |
//! | MP_SECURITY_6 | `positional` | `database`, `ftp`, `sftp` | Functions that take a user name and password as arguments 2 and 3. |
//! | MP_SECURITY_6 | `options` | `Username`, `Password` | Option and property names that hold credentials. |
//!
//! ## Examples
//!
//! ### Incorrect
//!
//! ```matlab
//! eval(cmd);
//! !backup.sh
//! opts = weboptions("CertificateFilename", "");
//! doc = xmlread(file);
//! password = "hunter2";
//! opts = weboptions("Username", "ann", "Password", "hunter2");
//! ```
//!
//! ### Correct
//!
//! ```matlab
//! result = fcn(x);
//! system("backup.sh");
//! opts = weboptions("CertificateFilename", "default");
//! doc = xmlread(file, "AllowDoctype", false);
//! [user, pw] = askLogin();
//! opts = weboptions("Username", user, "Password", pw);
//! ```

use malt_core::{Config, Rule};

use tree_sitter::Node;

use super::{Check, GroupEngine};

mod check_certificate;
mod check_constant_names;
mod check_credentials;
mod check_eval;
mod check_shell;
mod check_xml;

/// The checks of this group.
pub(crate) const CHECKS: &[Check] = &[
    Check {
        id: "MP_SECURITY_1",
        level: 3,
        run: check_eval::run,
    },
    Check {
        id: "MP_SECURITY_2",
        level: 3,
        run: check_shell::run,
    },
    Check {
        id: "MP_SECURITY_3",
        level: 4,
        run: check_certificate::run,
    },
    Check {
        id: "MP_SECURITY_4",
        level: 3,
        run: check_xml::run,
    },
    Check {
        id: "MP_SECURITY_5",
        level: 3,
        run: check_constant_names::run,
    },
    Check {
        id: "MP_SECURITY_6",
        level: 3,
        run: check_credentials::run,
    },
];

/// Call `f` on every node under `root`, in document order, without recursion.
pub(crate) fn for_each_node<'t>(root: Node<'t>, mut f: impl FnMut(Node<'t>)) {
    let mut cursor = root.walk();
    'outer: loop {
        f(cursor.node());
        if cursor.goto_first_child() {
            continue;
        }
        loop {
            if cursor.goto_next_sibling() {
                continue 'outer;
            }
            if !cursor.goto_parent() {
                break 'outer;
            }
        }
    }
}

/// The arguments of a `function_call`, without comments.
pub(crate) fn args_of(call: Node<'_>) -> Vec<Node<'_>> {
    let mut cursor = call.walk();
    let Some(arguments) = call.children(&mut cursor).find(|c| c.kind() == "arguments") else {
        return Vec::new();
    };
    let mut cursor = arguments.walk();
    arguments
        .named_children(&mut cursor)
        .filter(|c| c.kind() != "comment")
        .collect()
}

/// Whether `node` is a string or char literal.
pub(crate) fn is_string(node: Node<'_>) -> bool {
    node.kind() == "string"
}

/// The text of a string or char literal without its quotes.
pub(crate) fn string_value<'a>(node: Node<'_>, source: &'a str) -> Option<&'a str> {
    if node.kind() != "string" {
        return None;
    }
    let text = &source[node.start_byte()..node.end_byte()];
    (text.len() >= 2).then(|| &text[1..text.len() - 1])
}

/// Whether `node` is an empty string or char literal.
pub(crate) fn is_empty_string(node: Node<'_>, source: &str) -> bool {
    string_value(node, source).is_some_and(str::is_empty)
}

/// `Some(true)` for the literals `true` and `1`, `Some(false)` for `false` and `0`,
/// `None` for anything else.
pub(crate) fn is_literal_bool(node: Node<'_>, source: &str) -> Option<bool> {
    match &source[node.start_byte()..node.end_byte()] {
        "true" | "1" => Some(true),
        "false" | "0" => Some(false),
        _ => None,
    }
}

/// Factory constructor invoked by inventory.
pub fn from_config(config: &Config) -> Box<dyn Rule> {
    GroupEngine::boxed(
        "MP_SECURITY_ENGINE",
        "MonkeyProof security rules",
        CHECKS,
        config,
    )
}

inventory::submit!(crate::RuleRegistration::new(
    "MP_SECURITY_ENGINE",
    from_config
));

/// The last field of a `field_expression` (`c` in `a.b.c`).
pub(crate) fn last_field(expr: Node<'_>) -> Option<Node<'_>> {
    let mut cursor = expr.walk();
    let last = expr.children_by_field_name("field", &mut cursor).last();
    last
}
