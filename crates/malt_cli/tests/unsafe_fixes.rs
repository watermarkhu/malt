//! Unsafe fixes are suggestions: `--fix` must leave them alone and only
//! `--fix --unsafe-fixes` may apply them.
//!
//! Each case is a rewrite that is right in the common case and wrong in some
//! other (a different result class, `&&` on arrays, NaN, ...). If a fix like
//! this ever becomes safe by default again, this test says so.

use std::path::Path;

use malt_core::{fix_source, Config, FixMode, Linter, RuleRegistry};

fn linter() -> Linter {
    let config = Config::from_toml("[lint]\npreset = \"all\"\n").unwrap();
    let registry = RuleRegistry::new(malt_rules::active_rules(&config), &config);
    Linter::new(registry)
}

fn fixed(source: &str, mode: FixMode) -> String {
    let mut linter = linter();
    fix_source(&mut linter, source, Path::new("case.m"), mode)
        .unwrap()
        .source
}

/// (source, text the unsafe rewrite introduces)
const UNSAFE: &[(&str, &str)] = &[
    // str2num evaluates expressions, str2double returns NaN for them.
    ("n = str2num('1 2');\ndisp(n);\n", "str2double"),
    // fliplr(sort(x)) flips along the second dimension even for a column.
    ("y = fliplr(sort(m));\ndisp(y);\n", "sort(m, 'descend')"),
    // && errors on non-scalar operands, & does not.
    ("if a & b\n  disp(1);\nend\n", "a && b"),
    ("if a | b\n  disp(1);\nend\n", "a || b"),
    // strfind and contains differ for cell and non-text input.
    (
        "y = isempty(strfind(s, 'a'));\ndisp(y);\n",
        "~contains(s, 'a')",
    ),
    // lower/upper reject non-text input that strcmpi accepts.
    (
        "y = strcmpi(lower(s), 'abc');\ndisp(y);\n",
        "strcmpi(s, 'abc')",
    ),
    // f(x) == true equals f(x) only when f returns a logical.
    ("if numel(A) == true\n  disp(1);\nend\n", "if numel(A)\n"),
];

#[test]
fn safe_mode_leaves_unsafe_rewrites_alone_and_all_mode_applies_them() {
    // A big stack, as the CLI uses (test threads get 2 MiB).
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
            for (source, rewrite) in UNSAFE {
                let safe = fixed(source, FixMode::Safe);
                assert!(
                    !safe.contains(rewrite),
                    "{source:?}: --fix must not apply the unsafe rewrite {rewrite:?}, got {safe:?}"
                );
                let all = fixed(source, FixMode::All);
                assert!(
                    all.contains(rewrite),
                    "{source:?}: --unsafe-fixes should apply {rewrite:?}, got {all:?}"
                );
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

/// Rewrites that were removed outright because no equivalent exists: neither
/// mode may touch these.
const NO_REWRITE: &[(&str, &str)] = &[
    ("r = deblank(s);\ndisp(r);\n", "strtrim"),
    ("r = strtrim(s);\ndisp(r);\n", "strip"),
    ("p = findstr(a, b);\ndisp(p);\n", "contains"),
    ("b = im2bw(i);\ndisp(b);\n", "imbinarize"),
    ("if isempty(x == 1)\n  disp(1);\nend\n", "any("),
    ("if strcmp(upper(s), 'ABC')\n  disp(1);\nend\n", "strcmp(s,"),
    ("parfor i = 1:2:9\n  disp(i);\nend\n", "1:9"),
    ("t = sprintf('%f', pi);\ndisp(t);\n", "num2str"),
    // No Code Analyzer check describes these; the old arithmetic/isa checks were removed.
    ("y = x * 1;\ndisp(y);\n", "y = x;"),
    ("if isa(x, 'double')\n  disp(1);\nend\n", "isnumeric(x)"),
];

#[test]
fn removed_rewrites_stay_removed() {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
            for (source, rewrite) in NO_REWRITE {
                for mode in [FixMode::Safe, FixMode::All] {
                    let out = fixed(source, mode);
                    assert!(
                        !out.contains(rewrite),
                        "{source:?} ({mode:?}) must not be rewritten to use {rewrite:?}, got {out:?}"
                    );
                }
            }
        })
        .unwrap()
        .join()
        .unwrap();
}
