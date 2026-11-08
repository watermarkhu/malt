//! Fix application helpers shared by the CLI and the wasm binding.
//!
//! The rules at the heart of malt report fixes independently, so the applier
//! has to make them compose:
//!
//! - a [`Fix`] with additional edits is applied **atomically**: all of its edits
//!   or none;
//! - identical edits proposed by different checks are applied once (two checks
//!   inserting the same `;` at the same byte must not produce `;;`);
//! - conflicting fixes are skipped, the first one in diagnostic order wins;
//! - only [`Applicability::Safe`] fixes are applied unless the caller opts in to
//!   unsafe ones;
//! - [`fix_source`] repeats lint-and-fix until the source stops changing, as a
//!   fix can enable another one and a skipped, conflicting fix gets its turn.

use std::path::Path;

use crate::diagnostic::{Applicability, Diagnostic, Fix};
use crate::linter::{LintError, Linter};

/// Which fixes to apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FixMode {
    /// Only fixes that preserve the meaning of the program ([`Applicability::Safe`]).
    #[default]
    Safe,
    /// Safe fixes and unsafe ones ([`Applicability::Unsafe`]).
    All,
}

impl FixMode {
    fn allows(self, fix: &Fix) -> bool {
        match self {
            FixMode::Safe => fix.applicability == Applicability::Safe,
            FixMode::All => true,
        }
    }
}

/// Apply the safe fixes of `diagnostics` to `source`.
///
/// Shorthand for [`apply_fixes_with`] with [`FixMode::Safe`].
pub fn apply_fixes(source: &str, diagnostics: &[Diagnostic]) -> String {
    apply_fixes_with(source, diagnostics, FixMode::Safe).0
}

/// Two edits conflict when their byte ranges overlap, or when two insertions
/// target the same offset with different text (their order would be ambiguous).
fn conflicts(a: &Fix, b: &Fix) -> bool {
    if a.byte_range == b.byte_range && a.replacement == b.replacement {
        return false; // identical: harmless, the applier collapses them
    }
    let (a0, a1) = (a.byte_range.start, a.byte_range.end);
    let (b0, b1) = (b.byte_range.start, b.byte_range.end);
    if a0 == a1 && b0 == b1 {
        return a0 == b0;
    }
    a0 < b1 && b0 < a1
}

/// Apply the fixes of `diagnostics` selected by `mode` to `source`.
///
/// Returns the new source and the number of fixes that were applied. Fixes are
/// considered in diagnostic order; a fix that conflicts with one already
/// accepted is skipped (and picked up by the next pass of [`fix_source`]).
pub fn apply_fixes_with(
    source: &str,
    diagnostics: &[Diagnostic],
    mode: FixMode,
) -> (String, usize) {
    // Each diagnostic's fix is a group of edits that stands or falls together.
    let mut accepted: Vec<&Fix> = Vec::new();
    let mut applied = 0;

    'groups: for diag in diagnostics {
        let Some(fix) = diag.fix.as_ref().filter(|f| mode.allows(f)) else {
            continue;
        };
        let group: Vec<&Fix> = std::iter::once(fix).chain(&fix.additional_edits).collect();

        // Out-of-bounds or non-boundary edits would corrupt (or panic on) the text.
        for edit in &group {
            let r = &edit.byte_range;
            if r.start > r.end
                || r.end > source.len()
                || !source.is_char_boundary(r.start)
                || !source.is_char_boundary(r.end)
            {
                eprintln!(
                    "malt: warning: skipping invalid fix at bytes {}..{} (source length {})",
                    r.start,
                    r.end,
                    source.len()
                );
                continue 'groups;
            }
        }
        // The group's own edits must not collide with each other either.
        for (i, a) in group.iter().enumerate() {
            if group[i + 1..].iter().any(|b| conflicts(a, b)) {
                continue 'groups;
            }
        }
        // Skip the whole group if any of its edits conflicts with an accepted one.
        if group
            .iter()
            .any(|edit| accepted.iter().any(|other| conflicts(edit, other)))
        {
            continue;
        }

        let before = accepted.len();
        for edit in group {
            let duplicate = accepted
                .iter()
                .any(|o| o.byte_range == edit.byte_range && o.replacement == edit.replacement);
            if !duplicate {
                accepted.push(edit);
            }
        }
        // A group whose edits were all already present adds nothing: it is a
        // duplicate of a fix proposed by another check, not a new fix.
        if accepted.len() > before {
            applied += 1;
        }
    }

    // Apply back to front so earlier offsets stay valid. For equal starts the
    // longer edit goes first, so an insertion lands in front of a replacement.
    accepted.sort_by_key(|e| std::cmp::Reverse((e.byte_range.start, e.byte_range.end)));
    let mut result = source.to_string();
    for edit in accepted {
        result.replace_range(edit.byte_range.clone(), &edit.replacement);
    }
    (result, applied)
}

/// The outcome of [`fix_source`].
#[derive(Debug)]
pub struct FixOutcome {
    /// The fixed source (equal to the input when nothing was fixed).
    pub source: String,
    /// Number of fixes applied over all passes.
    pub applied: usize,
    /// Number of lint-and-fix passes that changed the source.
    pub passes: usize,
    /// Diagnostics that remain after fixing.
    pub remaining: Vec<Diagnostic>,
}

/// Upper bound on lint-and-fix passes; fixes converge long before this, the
/// bound only guards against two fixes undoing each other forever.
const MAX_FIX_PASSES: usize = 10;

/// Lint `source` and apply fixes until it stops changing.
///
/// A fix can enable another one (removing a statement exposes an unused
/// variable) and fixes that conflicted in one pass are retried on the next, so
/// a single pass can leave fixable problems behind. The returned
/// [`FixOutcome::remaining`] holds what is left, computed on the final source.
pub fn fix_source(
    linter: &mut Linter,
    source: &str,
    file_path: &Path,
    mode: FixMode,
) -> Result<FixOutcome, LintError> {
    let mut current = source.to_string();
    let mut applied = 0;
    let mut passes = 0;

    loop {
        let diagnostics = linter.lint(&current, file_path)?;
        if passes == MAX_FIX_PASSES {
            return Ok(FixOutcome {
                source: current,
                applied,
                passes,
                remaining: diagnostics,
            });
        }
        let (fixed, n) = apply_fixes_with(&current, &diagnostics, mode);
        if n == 0 || fixed == current {
            return Ok(FixOutcome {
                source: current,
                applied,
                passes,
                remaining: diagnostics,
            });
        }
        applied += n;
        passes += 1;
        current = fixed;
    }
}

/// Compute the 1-indexed `(line, column)` of a byte offset in `source`.
pub fn position_of(source: &str, byte: usize) -> (usize, usize) {
    // Clamp to the source length: some rules compute end positions that point
    // just past the last byte (e.g. an insertion fix at end-of-file), which is
    // valid for the fix but not a valid slice index.
    let byte = byte.min(source.len());
    let prefix = &source[..byte];
    let line = prefix.bytes().filter(|b| *b == b'\n').count() + 1;
    let line_start = prefix.rfind('\n').map_or(0, |p| p + 1);
    (line, byte - line_start + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::Severity;

    fn diag(fix: Fix) -> Diagnostic {
        Diagnostic {
            rule_id: "T",
            message: String::new(),
            severity: Severity::Info,
            byte_range: 0..0,
            line: 1,
            column: 1,
            fix: Some(fix),
        }
    }

    #[test]
    fn identical_insertions_from_two_checks_apply_once() {
        // The `x = 1;;` bug: NOSEMI and NOPTS both inserted `;` at the same byte.
        let (out, applied) = apply_fixes_with(
            "x = 1",
            &[diag(Fix::insert(5, ";")), diag(Fix::insert(5, ";"))],
            FixMode::Safe,
        );
        assert_eq!(out, "x = 1;");
        assert_eq!(applied, 1);
    }

    #[test]
    fn different_insertions_at_the_same_offset_conflict() {
        let out = apply_fixes(
            "ab",
            &[diag(Fix::insert(1, "X")), diag(Fix::insert(1, "Y"))],
        );
        assert_eq!(out, "aXb", "the first fix wins, the second is skipped");
    }

    #[test]
    fn overlapping_replacements_keep_the_first() {
        let out = apply_fixes(
            "abcdef",
            &[diag(Fix::new(1..4, "X")), diag(Fix::new(3..5, "Y"))],
        );
        assert_eq!(out, "aXef");
    }

    #[test]
    fn adjacent_edits_do_not_conflict() {
        let out = apply_fixes(
            "abcd",
            &[diag(Fix::new(0..2, "X")), diag(Fix::new(2..4, "Y"))],
        );
        assert_eq!(out, "XY");
    }

    #[test]
    fn insertion_before_a_replacement_at_the_same_start() {
        let out = apply_fixes(
            "abc",
            &[diag(Fix::new(1..2, "X")), diag(Fix::insert(1, "<"))],
        );
        assert_eq!(out, "a<Xc");
    }

    #[test]
    fn multi_edit_fix_is_atomic() {
        // The second group conflicts with the first on its additional edit: it
        // must be skipped as a whole, not half-applied.
        let first = Fix::new(0..1, "A");
        let second = Fix::with_additional(2..3, "C", vec![Fix::new(0..1, "Z")]);
        let out = apply_fixes("abc", &[diag(first), diag(second)]);
        assert_eq!(out, "Abc");
    }

    #[test]
    fn multi_edit_fix_applies_all_edits() {
        let fix = Fix::with_additional(2..3, "C", vec![Fix::new(0..1, "A")]);
        assert_eq!(apply_fixes("abc", &[diag(fix)]), "AbC");
    }

    #[test]
    fn unsafe_fixes_need_opt_in() {
        let d = [diag(Fix::new(0..1, "X").unsafe_fix())];
        assert_eq!(apply_fixes("abc", &d), "abc");
        assert_eq!(apply_fixes_with("abc", &d, FixMode::All).0, "Xbc");
    }

    #[test]
    fn invalid_ranges_are_skipped_not_applied() {
        assert_eq!(apply_fixes("abc", &[diag(Fix::new(2..9, "X"))]), "abc");
        // Inside a multi-byte character.
        assert_eq!(apply_fixes("é", &[diag(Fix::new(1..1, "X"))]), "é");
    }

    #[test]
    fn position_of_clamps() {
        assert_eq!(position_of("a\nb", 99), (2, 2));
    }
}
