//! # Case conversion in `strcmp` and `strncmp`
//!
//! [`strcmp_case`] tells how MATLAB sees a `strcmp` or `strncmp` call whose
//! arguments convert case with `lower` or `upper`. Each argument is one of:
//! a direct `lower(...)` or `upper(...)` call, a variable that holds one, a
//! single-quoted literal (or a variable that holds one), or something unknown.
//!
//! Probed (MATLAB R2026b): a conversion against a string of the other case is
//! STCUL, since the comparison fails. The same conversion against a string of
//! the matching case, or against an unknown string, is STCI (STNCI for
//! `strncmp`). A literal without letters makes the conversion unnecessary
//! (STLOW), and two literals are silent whatever their case.

use tree_sitter::Node;

use super::elementwise::call_name;
use super::reaching::{reaching, Binding};

/// How a `strcmp` or `strncmp` call compares case.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StrcmpCase {
    /// The strings differ in case, so the comparison fails (STCUL).
    Mismatch,
    /// A case conversion where a case-insensitive comparison would do (STCI, STNCI).
    Insensitive,
}

/// What one argument of the comparison is known to be.
enum Side<'s> {
    /// A direct `lower(...)` (`false`) or `upper(...)` (`true`) call, with the
    /// literal it converts when its argument is a known string.
    Call(bool, Option<&'s str>),
    /// A variable whose value is a `lower` or `upper` call.
    Var(bool),
    /// A single-quoted string literal, without its quotes.
    Literal(&'s str),
    /// Anything else.
    Unknown,
}

/// The case verdict for a `strcmp` or `strncmp` call, or `None` when MATLAB
/// reports nothing for it (no conversion, a literal without letters, or two
/// literals).
pub(crate) fn strcmp_case(call: Node, source: &str) -> Option<StrcmpCase> {
    if call.kind() != "function_call" {
        return None;
    }
    if !matches!(call_name(call, source), Some("strcmp" | "strncmp")) {
        return None;
    }
    let args = call_args(call);
    let (first, second) = (args.first()?, args.get(1)?);
    let (a, b) = (side(*first, source), side(*second, source));
    match (a, b) {
        (Side::Call(p, _), Side::Call(q, _)) => Some(if p == q {
            StrcmpCase::Insensitive
        } else {
            StrcmpCase::Mismatch
        }),
        (Side::Call(upper, Some(value)), Side::Literal(s))
        | (Side::Literal(s), Side::Call(upper, Some(value))) => known_verdict(value, s, upper),
        (Side::Call(upper, _), Side::Literal(s)) | (Side::Literal(s), Side::Call(upper, _)) => {
            if !s.chars().any(char::is_alphabetic) {
                None
            } else if conflicts(s, upper) {
                Some(StrcmpCase::Mismatch)
            } else {
                Some(StrcmpCase::Insensitive)
            }
        }
        (Side::Var(upper), Side::Literal(s)) | (Side::Literal(s), Side::Var(upper)) => {
            conflicts(s, upper).then_some(StrcmpCase::Mismatch)
        }
        (Side::Call(_, _), Side::Unknown) | (Side::Unknown, Side::Call(_, _)) => {
            Some(StrcmpCase::Insensitive)
        }
        _ => None,
    }
}

/// The verdict for a conversion of a known string against a literal (probed, MATLAB
/// R2026b). A conversion that changes nothing and matches the literal is silent
/// (`b = 'x'; strcmp(lower(b), 'x')`). A conversion that changes the value into the
/// literal is STCI. A literal that differs from the unchanged value only in case is
/// STCUL, but silent when the conversion changed the value.
fn known_verdict(value: &str, literal: &str, upper: bool) -> Option<StrcmpCase> {
    if !literal.chars().any(char::is_alphabetic) {
        return None;
    }
    let converted = if upper {
        value.to_uppercase()
    } else {
        value.to_lowercase()
    };
    if converted == value {
        if converted == literal {
            None
        } else if converted.eq_ignore_ascii_case(literal) {
            Some(StrcmpCase::Mismatch)
        } else {
            Some(StrcmpCase::Insensitive)
        }
    } else if converted == literal {
        Some(StrcmpCase::Insensitive)
    } else if converted.eq_ignore_ascii_case(literal) {
        None
    } else {
        Some(StrcmpCase::Insensitive)
    }
}

/// Whether the literal `s` differs in case from a conversion's output: a
/// `upper` output has no lowercase letters, a `lower` output no uppercase ones.
fn conflicts(s: &str, upper: bool) -> bool {
    if upper {
        s.chars().any(char::is_lowercase)
    } else {
        s.chars().any(char::is_uppercase)
    }
}

/// Classifies one argument of the comparison.
fn side<'s>(arg: Node, source: &'s str) -> Side<'s> {
    match arg.kind() {
        "function_call" => match call_name(arg, source) {
            Some("lower") => Side::Call(false, call_text_arg(arg, source)),
            Some("upper") => Side::Call(true, call_text_arg(arg, source)),
            _ => Side::Unknown,
        },
        "string" => literal(arg, source).map_or(Side::Unknown, Side::Literal),
        "identifier" => {
            let name = &source[arg.byte_range()];
            let r = reaching(arg, source, name);
            if r.indexed_since {
                return Side::Unknown;
            }
            match r.binding {
                Binding::Value(rhs) => match rhs.kind() {
                    "function_call" => match call_name(rhs, source) {
                        Some("lower") => Side::Var(false),
                        Some("upper") => Side::Var(true),
                        _ => Side::Unknown,
                    },
                    "string" => literal(rhs, source).map_or(Side::Unknown, Side::Literal),
                    _ => Side::Unknown,
                },
                _ => Side::Unknown,
            }
        }
        _ => Side::Unknown,
    }
}

/// The known string a `lower` or `upper` call converts: its first argument, a
/// single-quoted literal or a variable whose only reaching value is one.
fn call_text_arg<'s>(call: Node, source: &'s str) -> Option<&'s str> {
    let first = *call_args(call).first()?;
    match first.kind() {
        "string" => literal(first, source),
        "identifier" => {
            let r = reaching(first, source, &source[first.byte_range()]);
            if r.indexed_since {
                return None;
            }
            match r.binding {
                Binding::Value(rhs) if rhs.kind() == "string" => literal(rhs, source),
                _ => None,
            }
        }
        _ => None,
    }
}

/// The value of a single-quoted literal (`'abc'`). A double-quoted string and a
/// literal with an escaped quote are not known here.
fn literal<'s>(node: Node, source: &'s str) -> Option<&'s str> {
    let text = &source[node.byte_range()];
    let inner = text.strip_prefix('\'')?.strip_suffix('\'')?;
    (!inner.contains('\'')).then_some(inner)
}

/// The arguments of a call, in order.
fn call_args(call: Node) -> Vec<Node> {
    let mut cursor = call.walk();
    let Some(arguments) = call.children(&mut cursor).find(|c| c.kind() == "arguments") else {
        return Vec::new();
    };
    let mut inner = arguments.walk();
    arguments.named_children(&mut inner).collect()
}
