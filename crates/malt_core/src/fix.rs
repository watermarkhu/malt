//! Fix application helpers shared by the CLI and the wasm binding.

use crate::diagnostic::Diagnostic;

/// Apply all available fixes to the source text.
///
/// Fixes are applied in reverse byte-offset order to preserve earlier offsets.
/// Overlapping fixes are detected and skipped to prevent corruption.
pub fn apply_fixes(source: &str, diagnostics: &[Diagnostic]) -> String {
    // Collect all edits (primary + additional) from diagnostics that have fixes.
    let mut edits: Vec<&crate::diagnostic::Fix> = Vec::new();
    for diag in diagnostics {
        if let Some(ref fix) = diag.fix {
            edits.push(fix);
            for additional in &fix.additional_edits {
                edits.push(additional);
            }
        }
    }

    // Sort edits by byte_range start in reverse order so applying them
    // back-to-front doesn't invalidate earlier offsets.
    edits.sort_by_key(|e| std::cmp::Reverse(e.byte_range.start));

    let mut result = source.to_string();
    let mut last_edit_start = usize::MAX;

    for fix in edits {
        // Skip overlapping edits: if this fix's range overlaps with the
        // previously applied fix, skip it to prevent corruption.
        if fix.byte_range.end > last_edit_start {
            eprintln!(
                "malt: warning: skipping overlapping fix at bytes {}..{} (conflicts with edit at {})",
                fix.byte_range.start, fix.byte_range.end, last_edit_start
            );
            continue;
        }
        // Guard against out-of-bounds fix ranges (defensive: rules compute
        // offsets against the original source, which is the same string here).
        if fix.byte_range.start > result.len() || fix.byte_range.end > result.len() {
            eprintln!(
                "malt: warning: skipping out-of-bounds fix at bytes {}..{} (source length {})",
                fix.byte_range.start,
                fix.byte_range.end,
                result.len()
            );
            continue;
        }
        result.replace_range(fix.byte_range.clone(), &fix.replacement);
        last_edit_start = fix.byte_range.start;
    }

    result
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
