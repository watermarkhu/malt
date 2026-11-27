//! # MP_NAMING_ENGINE: MonkeyProof naming conventions
//!
//! ```malt
//! id = "MP_NAMING_ENGINE"
//! title = "MonkeyProof naming conventions"
//! category = "monkeyproof"
//! severity = "info"
//! fix = false
//! slug = "monkeyproof-naming"
//! icon = "lucide/type"
//! note = "Not part of MATLAB's Code Analyzer. Off unless the monkeyproof preset or category is on."
//! ```
//!
//! ## Rule
//!
//! The checks of the naming conventions chapter of the
//! [MonkeyProof Coding Standard for MATLAB](https://doc.monkeyproofsolutions.nl/code-checker-for-matlab/monkeyproof-coding-standard-for-matlab/naming-conventions/naming-conventions.html) that can be checked automatically.
//!
//! The set is off by default. Switch it on with `preset = "monkeyproof"` or with
//! `[lint.categories] monkeyproof = "warn"`, and tune one check in its own table, for
//! example `[lint.rules.MP_NAMING_20] max_variable_length = 40`. Each check is limited to
//! what one file shows. A name is judged where the file declares it: a variable once per
//! function, at its first assignment, input, output or loop header.
//!
//! Parameters, with their defaults:
//!
//! - `MP_NAMING_2`: `min_length = 3`, `first_letter = "i"`. The outermost loop of a
//!   function should start with `first_letter` and each nested loop with the next letter.
//!   A loop that is not nested in another for loop counts as outermost, and so does the
//!   first loop of every function. Loops separated by other statements still count as
//!   nested when one contains the other.
//! - `MP_NAMING_4`: `min_parent_length = 3`. Shorter parent names are not compared.
//!   Only fields that an assignment writes (`s.name = 1`) and properties are checked, not
//!   fields that code reads. The comparison ignores case, and the parent is the name in
//!   front of the field in the chain (`b` in `a.b.c`).
//! - `MP_NAMING_14`: `additional_names = []`, `allowed_names = []`. The check knows a list
//!   of core MATLAB functions, not the functions of every toolbox, and not the other files
//!   of the project, so shadowing of project code is not reported. Methods are not
//!   reported either, since a class may define `disp` or `plot` for itself.
//! - `MP_NAMING_15`: `prefixes = ["temp", "my"]` and `names` (placeholders such as
//!   `testFunction`). A prefix counts when it is followed by an uppercase letter, a
//!   digit, an underscore or the end of the name, so `temperature` is fine and
//!   `tempValue` is not.
//! - `MP_NAMING_18`: `include_optional = false`. `addOptional` declares a positional
//!   argument, not a name-value pair, so it is only checked when asked for. Calls are
//!   only checked on variables that the file assigns `inputParser` to. In an `arguments`
//!   block only the dotted names of a struct input (`options.Name`) are name-value pairs.
//!   A name passes when it starts with an uppercase letter and has letters only.
//! - `MP_NAMING_19`: no parameters. A file is a unit test when its class derives from a
//!   `TestCase` or from a class whose name ends in `Test`, or when it calls
//!   `functiontests`. Scripts are never reported, since a test script looks like any other
//!   script. Methods are not reported. A file name ending in `Test` is reported at the
//!   first class or function of the file unless that name is reported already.
//! - `MP_NAMING_20`: `max_variable_length = 63`, `min_function_length = 3`,
//!   `max_function_length = 63`. Constructors are not function names, they are the class
//!   name.
//!
//! ## Check IDs
//!
//! | Check ID | Severity | Fix | Description |
//! | -------- | -------- | --- | ----------- |
//! | MP_NAMING_2 | info | no | A for loop variable is shorter than 3 characters or does not start with the letter of its nesting depth (i, j, k and so on). |
//! | MP_NAMING_4 | info | no | A struct field or property name repeats the name of its parent. |
//! | MP_NAMING_14 | warning | no | A variable or function has the name of a MATLAB function. |
//! | MP_NAMING_15 | info | no | A class, function, variable, property or field has a placeholder name such as `temp...`, `my...` or `testFunction`. |
//! | MP_NAMING_18 | info | no | A name-value pair of an `arguments` block or an `inputParser` is not UpperCamelCase or has digits. |
//! | MP_NAMING_19 | info | no | A class, function or file name ends in `Test` but is not a unit test. |
//! | MP_NAMING_20 | info | no | A variable name has more than 63 characters, or a function name has fewer than 3 or more than 63. |
//!
//! ## Examples
//!
//! ### Incorrect
//!
//! ```matlab
//! function result = scaleAll(values)
//!     result = zeros(size(values));
//!     for n = 1:numel(values)
//!         for m = 1:2
//!             result(n, m) = values(n) * 2;
//!         end
//!     end
//!     max = 10;
//!     tempScale = max;
//! end
//! ```
//!
//! ### Correct
//!
//! ```matlab
//! function result = scaleAll(values)
//!     result = zeros(size(values));
//!     for iValue = 1:numel(values)
//!         for jColumn = 1:2
//!             result(iValue, jColumn) = values(iValue) * 2;
//!         end
//!     end
//!     maxScale = 10;
//!     scaleLimit = maxScale;
//! end
//! ```

use malt_core::{Config, Rule};

use super::{Check, GroupEngine};

mod check_iterator;
mod check_length;
mod check_name_value;
mod check_prefix;
mod check_redundancy;
mod check_shadowing;
mod check_test_suffix;
mod decls;

/// The checks of this group.
pub(crate) const CHECKS: &[Check] = &[
    Check {
        id: "MP_NAMING_2",
        level: 8,
        run: check_iterator::run,
    },
    Check {
        id: "MP_NAMING_4",
        level: 8,
        run: check_redundancy::run,
    },
    Check {
        id: "MP_NAMING_14",
        level: 3,
        run: check_shadowing::run,
    },
    Check {
        id: "MP_NAMING_15",
        level: 8,
        run: check_prefix::run,
    },
    Check {
        id: "MP_NAMING_18",
        level: 8,
        run: check_name_value::run,
    },
    Check {
        id: "MP_NAMING_19",
        level: 8,
        run: check_test_suffix::run,
    },
    Check {
        id: "MP_NAMING_20",
        level: 8,
        run: check_length::run,
    },
];

/// Factory constructor invoked by inventory.
pub fn from_config(config: &Config) -> Box<dyn Rule> {
    GroupEngine::boxed(
        "MP_NAMING_ENGINE",
        "MonkeyProof naming conventions",
        CHECKS,
        config,
    )
}

inventory::submit!(crate::RuleRegistration::new(
    "MP_NAMING_ENGINE",
    from_config
));
