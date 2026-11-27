//! # MP_LAYOUT_ENGINE: MonkeyProof layout and comments
//!
//! ```malt
//! id = "MP_LAYOUT_ENGINE"
//! title = "MonkeyProof layout and comments"
//! category = "monkeyproof"
//! severity = "info"
//! fix = false
//! slug = "monkeyproof-layout"
//! icon = "lucide/align-left"
//! note = "Not part of MATLAB's Code Analyzer, and malt is not affiliated with MonkeyProof Solutions. Off unless the monkeyproof preset or category is on."
//! ```
//!
//! ## Rule
//!
//! The checks of the layout and comments chapter of the
//! [MonkeyProof Coding Standard for MATLAB](https://doc.monkeyproofsolutions.nl/code-checker-for-matlab/monkeyproof-coding-standard-for-matlab/layout/layout.html) that can be checked automatically.
//! The standard is a style guide of its own and not part of MATLAB's Code Analyzer, so the
//! set is off by default. Switch it on for a whole project with `preset = "monkeyproof"`, or
//! on top of another preset with:
//!
//! ```toml
//! [lint.categories]
//! monkeyproof = "warn"
//! ```
//!
//! Each check can be tuned or turned off by its own ID (`[lint.rules] MP_LAYOUT_8 = "off"`).
//! Comments and strings are located with the syntax tree, so a `%` inside a string, a `'`
//! that is a transpose, block comments and `...` continuations are told apart correctly.
//!
//! ## Check IDs
//!
//! | Check ID | Severity | Fix | Description |
//! | -------- | -------- | --- | ----------- |
//! | MP_LAYOUT_1 | info | no | Do not use `%{ %}` block comments. |
//! | MP_LAYOUT_2 | info | no | Indent the body of every function, also at the root of a file. |
//! | MP_LAYOUT_3 | info | no | Surround operators with spaces; no space after a unary minus; no spaces needed in `Name=Value`. |
//! | MP_LAYOUT_4 | info | no | Follow commas, semicolons and `if`/`while`/`for`/`switch`/`case` keywords with a space. |
//! | MP_LAYOUT_5 | info | no | Keep lines within `max_length` characters (default 120). |
//! | MP_LAYOUT_6 | info | no | Indent each nesting level by `indent` spaces (default 4). |
//! | MP_LAYOUT_7 | info | no | Comments must not carry `TODO`, `FIXME`, `HACK` or similar markers (parameter `words`). |
//! | MP_LAYOUT_8 | info | no | Align the equals signs of consecutive assignments. |
//! | MP_LAYOUT_9 | info | no | Separate the elements of a matrix or cell row with commas placed directly after the element. |
//! | MP_LAYOUT_11 | info | no | Combine `properties` or `methods` blocks with equal attributes and leave out default attribute values. |
//! | MP_LAYOUT_12 | info | no | No whitespace at the end of a line. |
//! | MP_LAYOUT_13 | info | no | No tab characters. |
//! | MP_LAYOUT_14 | info | no | Put all `properties` blocks before all `methods` blocks of a class. |
//!
//! ## Parameters
//!
//! | Check | Parameter | Default | Meaning |
//! | ----- | --------- | ------- | ------- |
//! | MP_LAYOUT_5 | `max_length` | `120` | Longest line allowed, in characters. |
//! | MP_LAYOUT_6 | `indent` | `4` | Spaces per nesting level. |
//! | MP_LAYOUT_7 | `words` | `["TODO", "FIXME", "HACK", "XXX", "KLUDGE"]` | Marker words, matched as whole words and case-sensitively. |
//!
//! ```toml
//! [lint.rules.MP_LAYOUT_5]
//! max_length = 100
//!
//! [lint.rules.MP_LAYOUT_7]
//! words = ["TODO", "FIXME", "temporary fix"]
//! ```
//!
//! ## Notes on individual checks
//!
//! `MP_LAYOUT_2` and `MP_LAYOUT_6` look at constructs that start their own line and at body
//! lines that start with spaces. One-line forms, tab indentation and code with a syntax error
//! in it are skipped. The first offending line of a body is reported, not every line.
//!
//! `MP_LAYOUT_3` leaves the colon inside the parentheses or braces of a call or index
//! (`x(2:end)`) alone, because the grammar cannot tell indexing from a call, but checks ranges
//! anywhere else (`1 : n`, `for i = 1 : n`, `[1 : 3]`), as the standard writes them.
//!
//! `MP_LAYOUT_8` compares the column of the `=` in a run of assignments on consecutive lines
//! and reports the ones left of the rightmost.
//!
//! ## Examples
//!
//! ### Incorrect
//!
//! ```matlab
//! function total = addUp(values)
//! total=0;
//! for k=1:numel(values)
//!   total = total+values(k); % TODO: handle NaN
//! end
//! end
//! ```
//!
//! ### Correct
//!
//! ```matlab
//! function total = addUp(values)
//!     total = 0;
//!     for k = 1 : numel(values)
//!         total = total + values(k);
//!     end
//! end
//! ```

use malt_core::{Config, Rule};

use super::{Check, GroupEngine};

mod check_block_comments;
mod check_comment_words;
mod check_duplicate_attributes;
mod check_equals_alignment;
mod check_function_indent;
mod check_indent_width;
mod check_line_length;
mod check_operator_spacing;
mod check_properties_order;
mod check_row_commas;
mod check_space_after;
mod check_tabs;
mod check_trailing_whitespace;
mod indentation;
mod text;

/// The checks of this group.
pub(crate) const CHECKS: &[Check] = &[
    Check {
        id: "MP_LAYOUT_1",
        level: 7,
        run: check_block_comments::run,
    },
    Check {
        id: "MP_LAYOUT_2",
        level: 10,
        run: check_function_indent::run,
    },
    Check {
        id: "MP_LAYOUT_3",
        level: 9,
        run: check_operator_spacing::run,
    },
    Check {
        id: "MP_LAYOUT_4",
        level: 9,
        run: check_space_after::run,
    },
    Check {
        id: "MP_LAYOUT_5",
        level: 10,
        run: check_line_length::run,
    },
    Check {
        id: "MP_LAYOUT_6",
        level: 9,
        run: check_indent_width::run,
    },
    Check {
        id: "MP_LAYOUT_7",
        level: 7,
        run: check_comment_words::run,
    },
    Check {
        id: "MP_LAYOUT_8",
        level: 9,
        run: check_equals_alignment::run,
    },
    Check {
        id: "MP_LAYOUT_9",
        level: 9,
        run: check_row_commas::run,
    },
    Check {
        id: "MP_LAYOUT_11",
        level: 9,
        run: check_duplicate_attributes::run,
    },
    Check {
        id: "MP_LAYOUT_12",
        level: 9,
        run: check_trailing_whitespace::run,
    },
    Check {
        id: "MP_LAYOUT_13",
        level: 9,
        run: check_tabs::run,
    },
    Check {
        id: "MP_LAYOUT_14",
        level: 9,
        run: check_properties_order::run,
    },
];

/// Factory constructor invoked by inventory.
pub fn from_config(config: &Config) -> Box<dyn Rule> {
    GroupEngine::boxed(
        "MP_LAYOUT_ENGINE",
        "MonkeyProof layout and comments",
        CHECKS,
        config,
    )
}

inventory::submit!(crate::RuleRegistration::new(
    "MP_LAYOUT_ENGINE",
    from_config
));

/// The findings of check `id` on `source`, with `toml` as the configuration. For tests.
#[cfg(test)]
pub(crate) fn hits(id: &str, source: &str, toml: &str) -> Vec<malt_core::Diagnostic> {
    super::lint(from_config, source, toml)
        .into_iter()
        .filter(|d| d.rule_id == id)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monkeyproof::lint;

    #[test]
    fn a_conforming_file_is_clean() {
        let source = "\
function total = addUp(values, varargin)
    total       = 0;
    scale       = 2;
    for k = 1 : numel(values)
        if values(k) > 0 && scale ~= 1
            total = total + scale * values(k);
        elseif isnan(values(k))
            continue
        else
            disp('it''s % not a comment');
        end
    end
    pair = [1, -2; 3, 4]';
    opts = struct(Name=1);
end
";
        let d = lint(from_config, source, "");
        assert!(d.is_empty(), "{d:?}");
    }

    #[test]
    fn every_check_survives_broken_code() {
        for source in [
            "function f(\n  x = [1 2\n if (\n%{\n",
            "classdef A\n properties (\n methods\n function\n",
            "x = 'unterminated\n\t y = {1 2\n",
            "",
        ] {
            let _ = lint(from_config, source, "");
        }
    }
}
