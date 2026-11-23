//! Resource limits: the `[lint.limits]` table of `.malt.toml`.
//!
//! A pathological input (tens of thousands of nested brackets, a 50 MB file, a
//! single line of a million numbers) can make an analyzer use gigabytes of memory
//! or hours of time. These limits stop such input before it is read or parsed and
//! report it under malt's own diagnostic IDs ([`MALT_TOO_BIG`], [`MALT_TOO_DEEP`]),
//! which are not MATLAB messages.
//!
//! Every limit is a positive number the user may raise or lower; none can be
//! switched off. The defaults come from measurements (see `docs/VERIFY.md`,
//! "Resource limits"): memory and time grow with the *size* of a file (about 180
//! bytes of memory per input byte for ordinary statements, 500 for one huge
//! matrix), while nesting is cheap to parse (about 0.4 KB per level).
//!
//! | key | native default | wasm default |
//! |-----|----------------|--------------|
//! | `max_file_size` | 5 MiB | 2 MiB |
//! | `max_tree_depth` | 1,000 | 200 |
//! | `max_nesting` | 10,000 | 1,000 |
//! | `max_threads` | number of cores | 1 |

use crate::config::ConfigError;
use crate::diagnostic::{Diagnostic, Severity};

/// Diagnostic ID for a file larger than `max_file_size`.
pub const MALT_TOO_BIG: &str = "MALT_TOO_BIG";
/// Diagnostic ID for code nested deeper than `max_tree_depth` or `max_nesting`.
pub const MALT_TOO_DEEP: &str = "MALT_TOO_DEEP";

/// The keys of `[lint.limits]`.
pub const LIMIT_KEYS: &[&str] = &[
    "max_file_size",
    "max_tree_depth",
    "max_nesting",
    "max_threads",
];

/// Default `max_file_size` of the native build: 5 MiB.
pub const DEFAULT_MAX_FILE_SIZE: u64 = 5 * 1024 * 1024;
/// Default `max_tree_depth` of the native build.
pub const DEFAULT_MAX_TREE_DEPTH: usize = 1_000;
/// Default `max_nesting` of the native build.
pub const DEFAULT_MAX_NESTING: usize = 10_000;

/// Default `max_file_size` of the wasm build: 2 MiB.
pub const WASM_MAX_FILE_SIZE: u64 = 2 * 1024 * 1024;
/// Default `max_tree_depth` of the wasm build. Raising it can trap the module: a
/// chain of about 750 binary operators overflows the stack in V8.
pub const WASM_MAX_TREE_DEPTH: usize = 200;
/// Default `max_nesting` of the wasm build.
pub const WASM_MAX_NESTING: usize = 1_000;

/// The limits a linter works with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// Largest file, in bytes, that is read and analyzed.
    pub max_file_size: u64,
    /// Deepest syntax tree that is analyzed.
    pub max_tree_depth: usize,
    /// Most nested brackets (`(`, `[`, `{`) the input may hold; checked before parsing.
    pub max_nesting: usize,
    /// Most threads the process starts to analyze with, in total.
    pub max_threads: usize,
}

impl Limits {
    /// The defaults of the native build; `max_threads` is the number of cores.
    pub fn native() -> Self {
        Self {
            max_file_size: DEFAULT_MAX_FILE_SIZE,
            max_tree_depth: DEFAULT_MAX_TREE_DEPTH,
            max_nesting: DEFAULT_MAX_NESTING,
            max_threads: std::thread::available_parallelism().map_or(1, |n| n.get()),
        }
    }

    /// The defaults of the WebAssembly build: smaller, because a browser tab has
    /// a small stack and little memory. There are no threads.
    pub fn wasm() -> Self {
        Self {
            max_file_size: WASM_MAX_FILE_SIZE,
            max_tree_depth: WASM_MAX_TREE_DEPTH,
            max_nesting: WASM_MAX_NESTING,
            max_threads: 1,
        }
    }
}

impl Default for Limits {
    fn default() -> Self {
        Self::native()
    }
}

/// The `[lint.limits]` table as the user wrote it: a key left out is `None` and
/// takes the host's default (see [`LimitsConfig::resolve`]).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LimitsConfig {
    /// `max_file_size` in bytes.
    pub max_file_size: Option<u64>,
    /// `max_tree_depth`.
    pub max_tree_depth: Option<usize>,
    /// `max_nesting`.
    pub max_nesting: Option<usize>,
    /// `max_threads`.
    pub max_threads: Option<usize>,
    /// Keys of the table that name no limit (reported as warnings).
    pub unknown: Vec<String>,
}

impl LimitsConfig {
    /// Read the `[lint.limits]` table. A zero, negative or unparsable value is an
    /// error; an unknown key is kept in [`LimitsConfig::unknown`].
    pub fn from_table(table: &toml::Table) -> Result<Self, ConfigError> {
        let mut limits = Self::default();
        let mut keys: Vec<&String> = table.keys().collect();
        keys.sort();
        for key in keys {
            let value = &table[key];
            match key.as_str() {
                "max_file_size" => limits.max_file_size = Some(parse_size(key, value)?),
                "max_tree_depth" => limits.max_tree_depth = Some(parse_count(key, value)?),
                "max_nesting" => limits.max_nesting = Some(parse_count(key, value)?),
                "max_threads" => limits.max_threads = Some(parse_count(key, value)?),
                _ => limits.unknown.push(key.clone()),
            }
        }
        Ok(limits)
    }

    /// The limits to work with: each key the user set, the host's `defaults` for
    /// the others.
    pub fn resolve(&self, defaults: Limits) -> Limits {
        Limits {
            max_file_size: self.max_file_size.unwrap_or(defaults.max_file_size),
            max_tree_depth: self.max_tree_depth.unwrap_or(defaults.max_tree_depth),
            max_nesting: self.max_nesting.unwrap_or(defaults.max_nesting),
            max_threads: self.max_threads.unwrap_or(defaults.max_threads),
        }
    }
}

fn invalid(key: &str, detail: impl std::fmt::Display) -> ConfigError {
    ConfigError::InvalidLimit(format!("[lint.limits] {key}: {detail}"))
}

/// A positive whole number.
fn parse_count(key: &str, value: &toml::Value) -> Result<usize, ConfigError> {
    let n = value.as_integer().ok_or_else(|| {
        invalid(
            key,
            format!("expected a positive whole number, got {value}"),
        )
    })?;
    if n <= 0 {
        return Err(invalid(
            key,
            format!("must be greater than 0, got {n} (a limit cannot be switched off)"),
        ));
    }
    usize::try_from(n).map_err(|_| invalid(key, format!("{n} is too large")))
}

/// A size in bytes: a positive integer, or a string such as `"10 MiB"`.
fn parse_size(key: &str, value: &toml::Value) -> Result<u64, ConfigError> {
    match value {
        toml::Value::Integer(n) => {
            if *n <= 0 {
                return Err(invalid(
                    key,
                    format!("must be greater than 0, got {n} (a limit cannot be switched off)"),
                ));
            }
            Ok(*n as u64)
        }
        toml::Value::String(text) => parse_size_text(text).map_err(|why| invalid(key, why)),
        other => Err(invalid(
            key,
            format!("expected a number of bytes or a string such as \"10 MiB\", got {other}"),
        )),
    }
}

/// Parse `"1048576"`, `"512 KiB"`, `"10MiB"`, `"5 MB"`. Units (any case): `B`, `KiB`,
/// `MiB`, `GiB` (powers of 1024) and `KB`, `MB`, `GB` (powers of 1000); `K`, `M` and
/// `G` mean the 1024 units. The number must be a positive whole number.
pub fn parse_size_text(text: &str) -> Result<u64, String> {
    let text = text.trim();
    let split = text
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(text.len());
    let (digits, unit) = text.split_at(split);
    if digits.is_empty() {
        return Err(format!(
            "cannot read '{text}' as a size (expected a number with an optional unit, such as \"10 MiB\")"
        ));
    }
    let number: u64 = digits
        .parse()
        .map_err(|_| format!("the number in '{text}' is too large"))?;
    let factor: u64 = match unit.trim().to_ascii_lowercase().as_str() {
        "" | "b" => 1,
        "k" | "kib" => 1 << 10,
        "m" | "mib" => 1 << 20,
        "g" | "gib" => 1 << 30,
        "kb" => 1_000,
        "mb" => 1_000_000,
        "gb" => 1_000_000_000,
        other => {
            return Err(format!(
                "unknown size unit '{other}' in '{text}' (use B, KiB, MiB, GiB, KB, MB or GB)"
            ))
        }
    };
    let bytes = number
        .checked_mul(factor)
        .ok_or_else(|| format!("'{text}' is too large"))?;
    if bytes == 0 {
        return Err(format!(
            "'{text}' is zero: the size must be greater than 0 (a limit cannot be switched off)"
        ));
    }
    Ok(bytes)
}

/// Format a byte count for a message: `5242880` as `5 MiB (5242880 bytes)`.
pub fn format_size(bytes: u64) -> String {
    const UNITS: [(&str, u64); 3] = [("GiB", 1 << 30), ("MiB", 1 << 20), ("KiB", 1 << 10)];
    for (name, size) in UNITS {
        if bytes >= size && bytes.is_multiple_of(size) {
            return format!("{} {name} ({bytes} bytes)", bytes / size);
        }
    }
    format!("{bytes} bytes")
}

/// The MALT_TOO_BIG diagnostic for a file of `size` bytes over the limit `max`.
///
/// Not a MATLAB check: malt refuses the file to protect the memory of the machine.
pub fn too_big_diagnostic(size: u64, max: u64) -> Diagnostic {
    Diagnostic {
        rule_id: MALT_TOO_BIG,
        message: format!(
            "malt did not analyze this file: it is {} and malt's limit is {} \
             (max_file_size). To analyze it, raise max_file_size under [lint.limits] \
             in .malt.toml; larger files can use very large amounts of memory. \
             This is not a MATLAB message.",
            format_size(size),
            format_size(max)
        ),
        severity: Severity::Error,
        byte_range: 0..1,
        line: 1,
        column: 1,
        fix: None,
    }
}

/// The MALT_TOO_DEEP diagnostic for brackets nested beyond `max_nesting`, found
/// before parsing.
pub fn too_nested_diagnostic(max: usize) -> Diagnostic {
    Diagnostic {
        rule_id: MALT_TOO_DEEP,
        message: format!(
            "malt did not analyze this file: more than {max} brackets are nested \
             (max_nesting). To analyze it, raise max_nesting under [lint.limits] \
             in .malt.toml; deeply nested input can use very large amounts of memory. \
             This is not a MATLAB message."
        ),
        severity: Severity::Error,
        byte_range: 0..1,
        line: 1,
        column: 1,
        fix: None,
    }
}

/// Whether `source` nests more than `limit` brackets (`(`, `[` or `{`) at once.
///
/// A scan of the text in one pass and constant memory, made before the parse so a
/// nesting bomb is never parsed. Brackets in comments (`%` to the end of the line,
/// `...` continuations, `%{ ... %}` blocks, which nest), in `"..."` and `'...'`
/// strings are not counted. A `'` right after a name, number, closing bracket or
/// another transpose is a transpose, not a string. The scan only counts, it does not
/// match kinds: `(` closed by `]` still balances. Line ends reset strings, so an
/// unterminated string cannot hide the rest of the file. Block keywords (`if`,
/// `for`...) are not counted: `end` also closes an index (`x(end)`), so counting
/// them cannot be reliable; their depth is bounded by `max_tree_depth` after the
/// parse, which costs about 0.4 KB per level.
pub fn exceeds_nesting(source: &str, limit: usize) -> bool {
    let mut depth = 0usize;
    let mut block = 0usize;
    for line in source.as_bytes().split(|&c| c == b'\n') {
        let trimmed = line.trim_ascii();
        if block > 0 {
            if trimmed == b"%{" {
                block += 1;
            } else if trimmed == b"%}" {
                block -= 1;
            }
            continue;
        }
        if trimmed == b"%{" {
            block = 1;
            continue;
        }
        if scan_line(line, &mut depth, limit) {
            return true;
        }
    }
    false
}

/// Count the brackets of one line into `depth`; true when it passes `limit`.
fn scan_line(line: &[u8], depth: &mut usize, limit: usize) -> bool {
    let mut i = 0;
    // The last token ends a value (name, number, closing bracket, transpose), and
    // whitespace came after it.
    let mut after_value = false;
    let mut after_space = false;
    while i < line.len() {
        let c = line[i];
        match c {
            b'%' => return false,
            b'.' if line[i..].starts_with(b"...") => return false,
            b'.' if line.get(i + 1) == Some(&b'\'') => {
                // `.'`: the transpose operator.
                i += 2;
                after_value = true;
                after_space = false;
                continue;
            }
            b'"' => {
                i = skip_string(line, i, b'"');
                after_value = true;
                after_space = false;
                continue;
            }
            b'\'' => {
                if after_value && !after_space {
                    i += 1;
                } else {
                    i = skip_string(line, i, b'\'');
                }
                after_value = true;
                after_space = false;
                continue;
            }
            b'(' | b'[' | b'{' => {
                *depth += 1;
                if *depth > limit {
                    return true;
                }
                after_value = false;
                after_space = false;
            }
            b')' | b']' | b'}' => {
                *depth = depth.saturating_sub(1);
                after_value = true;
                after_space = false;
            }
            b' ' | b'\t' | b'\r' => after_space = true,
            c if c.is_ascii_alphanumeric() || c == b'_' => {
                after_value = true;
                after_space = false;
            }
            _ => {
                after_value = false;
                after_space = false;
            }
        }
        i += 1;
    }
    false
}

/// The index after the string that starts at `start` with `quote`. A doubled quote
/// is an escaped one; the string ends at the end of the line at the latest.
fn skip_string(line: &[u8], start: usize, quote: u8) -> usize {
    let mut i = start + 1;
    while i < line.len() {
        if line[i] == quote {
            if line.get(i + 1) == Some(&quote) {
                i += 2;
                continue;
            }
            return i + 1;
        }
        i += 1;
    }
    line.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nest(n: usize) -> String {
        format!("x = {}1{};\n", "(".repeat(n), ")".repeat(n))
    }

    #[test]
    fn the_boundary_is_exact() {
        assert!(!exceeds_nesting(&nest(5), 5));
        assert!(exceeds_nesting(&nest(6), 5));
        assert!(!exceeds_nesting("", 1));
        assert!(exceeds_nesting("(", 0));
    }

    #[test]
    fn all_three_kinds_count_together() {
        assert!(!exceeds_nesting("x = {[(1)]};", 3));
        assert!(exceeds_nesting("x = {[(1)]};", 2));
        // The scan counts, it does not match kinds.
        assert!(exceeds_nesting("x = ([{1}]);", 2));
    }

    #[test]
    fn depth_carries_over_lines() {
        let src = "x = [(\n(\n(\n1)])];\n";
        assert!(exceeds_nesting(src, 3));
        assert!(!exceeds_nesting(src, 4));
    }

    #[test]
    fn closed_brackets_do_not_add_up() {
        assert!(!exceeds_nesting(&"f(1);\n".repeat(10_000), 1));
        assert!(!exceeds_nesting("x = )))))((;", 2));
    }

    #[test]
    fn strings_and_comments_are_skipped() {
        let bombs = [
            "x = '((((((';",
            "x = \"((((((\";",
            "x = 'it''s ((((((';",
            "x = \"say \"\"((((((\"\"\";",
            "x = 1; % ((((((",
            "x = 1; ... ((((((",
            "x = [1 '((((((' 2];",
        ];
        for src in bombs {
            assert!(!exceeds_nesting(src, 1), "{src}");
        }
    }

    #[test]
    fn a_string_ends_at_its_closing_quote() {
        assert!(exceeds_nesting("x = '' ((((", 3));
        assert!(exceeds_nesting("x = 'a' ((((", 3));
        assert!(exceeds_nesting("x = \"a\" ((((", 3));
    }

    #[test]
    fn an_unterminated_string_ends_at_the_line() {
        assert!(exceeds_nesting("x = 'abc\ny = ((((", 3));
    }

    #[test]
    fn transpose_is_not_a_string() {
        for src in [
            "y = a'(((((;",
            "y = a.'(((((;",
            "y = a(1)'(((((;",
            "y = {1}'(((((;",
            "y = a''(((((;",
            "y = [a' b'](((((;",
            "y = 3'(((((;",
        ] {
            assert!(exceeds_nesting(src, 4), "{src}");
        }
        // After an operator or a bracket, a quote starts a string.
        assert!(!exceeds_nesting("y = ['(((((' ];", 1));
        assert!(!exceeds_nesting("y = ('(((((');", 1));
        assert!(!exceeds_nesting("y = {'(((((', 1};", 1));
        assert!(!exceeds_nesting("disp 'a(((((';", 1));
    }

    #[test]
    fn block_comments_are_skipped_and_nest() {
        let src = "%{\n((((((\n%{\n((((\n%}\n((((\n%}\nx = 1;\n";
        assert!(!exceeds_nesting(src, 1));
        // Code after the block counts again.
        let src = "%{\n((((((\n%}\nx = ((((1))));\n";
        assert!(exceeds_nesting(src, 3));
        assert!(!exceeds_nesting(src, 4));
        // `%{` that is not alone on its line is an ordinary comment.
        assert!(exceeds_nesting("x = 1; %{\n((((", 3));
        // With surrounding blanks it still opens a block.
        assert!(!exceeds_nesting("  %{  \n((((\n  %}  \n", 1));
        // An unclosed block swallows the rest of the file.
        assert!(!exceeds_nesting("%{\n((((((\n", 1));
    }

    #[test]
    fn carriage_returns_and_utf8_are_fine() {
        assert!(!exceeds_nesting("%{\r\n((((\r\n%}\r\nx = é;\r\n", 1));
        assert!(exceeds_nesting("x = é((((;\r\n", 3));
    }

    #[test]
    fn sizes_take_units() {
        assert_eq!(parse_size_text("1048576"), Ok(1_048_576));
        assert_eq!(parse_size_text("512 KiB"), Ok(512 * 1024));
        assert_eq!(parse_size_text("10MiB"), Ok(10 << 20));
        assert_eq!(parse_size_text("10 mib"), Ok(10 << 20));
        assert_eq!(parse_size_text("2 GiB"), Ok(2 << 30));
        assert_eq!(parse_size_text("5 MB"), Ok(5_000_000));
        assert_eq!(parse_size_text("3 KB"), Ok(3_000));
        assert_eq!(parse_size_text("7 M"), Ok(7 << 20));
        assert_eq!(parse_size_text("9 B"), Ok(9));
    }

    #[test]
    fn bad_sizes_are_errors() {
        for text in [
            "", "MiB", "-1 MiB", "1.5 MiB", "10 XiB", "0", "0 MiB", "1e3",
        ] {
            assert!(parse_size_text(text).is_err(), "{text}");
        }
        assert!(parse_size_text("99999999999999999999 B").is_err());
        assert!(parse_size_text("18446744073709551615 GiB").is_err());
    }

    #[test]
    fn size_formatting() {
        assert_eq!(format_size(5 << 20), "5 MiB (5242880 bytes)");
        assert_eq!(format_size(1500), "1500 bytes");
    }

    fn table(toml: &str) -> toml::Table {
        toml::from_str(toml).unwrap()
    }

    #[test]
    fn a_table_is_read_and_resolved() {
        let config = LimitsConfig::from_table(&table(
            "max_file_size = \"1 MiB\"\nmax_tree_depth = 50\nmax_threads = 2\n",
        ))
        .unwrap();
        let limits = config.resolve(Limits::wasm());
        assert_eq!(limits.max_file_size, 1 << 20);
        assert_eq!(limits.max_tree_depth, 50);
        assert_eq!(limits.max_threads, 2);
        // A key left out takes the host's default.
        assert_eq!(limits.max_nesting, WASM_MAX_NESTING);
        assert!(config.unknown.is_empty());
    }

    #[test]
    fn an_empty_table_keeps_the_defaults() {
        let config = LimitsConfig::from_table(&toml::Table::new()).unwrap();
        assert_eq!(config.resolve(Limits::wasm()), Limits::wasm());
        assert_eq!(Limits::native().max_file_size, 5 << 20);
        assert_eq!(Limits::native().max_tree_depth, 1_000);
        assert!(Limits::native().max_threads >= 1);
    }

    #[test]
    fn invalid_values_are_errors() {
        for bad in [
            "max_file_size = 0",
            "max_file_size = -5",
            "max_file_size = \"lots\"",
            "max_file_size = \"0 MiB\"",
            "max_file_size = true",
            "max_tree_depth = 0",
            "max_tree_depth = -1",
            "max_tree_depth = \"deep\"",
            "max_tree_depth = 1.5",
            "max_nesting = 0",
            "max_threads = 0",
            "max_threads = -2",
        ] {
            let err = LimitsConfig::from_table(&table(bad))
                .unwrap_err()
                .to_string();
            assert!(err.contains("[lint.limits]"), "{bad}: {err}");
        }
    }

    #[test]
    fn unknown_keys_are_kept() {
        let config = LimitsConfig::from_table(&table("max_file_sise = 1\nfoo = 2\n")).unwrap();
        assert_eq!(config.unknown, ["foo", "max_file_sise"]);
    }

    #[test]
    fn diagnostics_say_what_to_do() {
        let big = too_big_diagnostic(6 << 20, 5 << 20);
        assert_eq!(big.rule_id, "MALT_TOO_BIG");
        assert!(big.message.contains("max_file_size"));
        assert!(big.message.contains("5 MiB"));
        assert!(big.message.contains("not a MATLAB message"));
        assert_eq!(big.severity, Severity::Error);
        let deep = too_nested_diagnostic(10);
        assert_eq!(deep.rule_id, "MALT_TOO_DEEP");
        assert!(deep.message.contains("max_nesting"));
        assert_eq!((deep.line, deep.column), (1, 1));
    }
}
