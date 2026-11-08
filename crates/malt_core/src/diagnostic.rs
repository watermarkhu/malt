use std::ops::Range;

/// Severity level for a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Error => write!(f, "E"),
            Severity::Warning => write!(f, "W"),
            Severity::Info => write!(f, "I"),
        }
    }
}

/// A single atomic text edit: replace `byte_range` with `replacement`.
///
/// Following rumdl's model, a `Fix` may carry `additional_edits` that must be
/// applied atomically with the primary edit (e.g., inserting a declaration AND
/// replacing all usages of a magic number).
#[derive(Debug, Clone)]
pub struct Fix {
    /// Byte range in the source to replace.
    pub byte_range: Range<usize>,
    /// The replacement text (empty string = deletion).
    pub replacement: String,
    /// Additional edits that must be applied together with this one.
    pub additional_edits: Vec<Fix>,
}

impl Fix {
    /// Create a simple single-edit fix.
    pub fn new(byte_range: Range<usize>, replacement: impl Into<String>) -> Self {
        Self {
            byte_range,
            replacement: replacement.into(),
            additional_edits: Vec::new(),
        }
    }

    /// Create a fix with multiple coordinated edits.
    pub fn with_additional(
        byte_range: Range<usize>,
        replacement: impl Into<String>,
        additional: Vec<Fix>,
    ) -> Self {
        Self {
            byte_range,
            replacement: replacement.into(),
            additional_edits: additional,
        }
    }

    /// Convenience: create an insertion (zero-width replacement) at `offset`.
    pub fn insert(offset: usize, text: impl Into<String>) -> Self {
        Self::new(offset..offset, text)
    }
}

/// A lint diagnostic produced by a rule.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// The rule that produced this diagnostic (e.g., "M001").
    pub rule_id: &'static str,
    /// Human-readable message describing the issue.
    pub message: String,
    /// Severity level.
    pub severity: Severity,
    /// Byte range in source spanning the problematic region.
    pub byte_range: Range<usize>,
    /// 1-indexed line number.
    pub line: usize,
    /// 1-indexed column number.
    pub column: usize,
    /// Optional auto-fix.
    pub fix: Option<Fix>,
}
