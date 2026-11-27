//! # MP_FUNCTIONS_ENGINE: MonkeyProof higher level rules
//!
//! ```malt
//! id = "MP_FUNCTIONS_ENGINE"
//! title = "MonkeyProof higher level rules"
//! category = "monkeyproof"
//! severity = "info"
//! fix = false
//! slug = "monkeyproof-functions"
//! icon = "lucide/boxes"
//! note = "Not part of MATLAB's Code Analyzer, and malt is not affiliated with MonkeyProof Solutions. Off unless the monkeyproof preset or category is on."
//! ```
//!
//! ## Rule
//!
//! The checks of the higher level rules chapter of the
//! [MonkeyProof Coding Standard for MATLAB](https://doc.monkeyproofsolutions.nl/code-checker-for-matlab/monkeyproof-coding-standard-for-matlab/higher-level-rules/higher-level-rules.html) that can be checked automatically.
//!
//! The checks are off by default. Switch them on with `preset = "monkeyproof"`, or with
//! `[lint.categories] monkeyproof = "warn"`, or one by one under `[lint.rules]`.
//!
//! Each check reads one file. Where a rule needs more than that, the check does the part the
//! file shows and says so below. FCNS-23 (properties re-implemented from a superclass) and
//! FCNS-25 (method access differing from the superclass) are not implemented: they need the
//! superclass, and the path index of malt only tells whether a class is a handle class.
//!
//! ## Check IDs
//!
//! | Check ID | Severity | Fix | Description |
//! | -------- | -------- | --- | ----------- |
//! | MP_FCNS_1 | warning | no | A function has more code lines or a higher cyclomatic complexity than the limit for its kind (main, local, method, nested). |
//! | MP_FCNS_2 | info | no | A function or method declaration has more than five inputs or more than five outputs. |
//! | MP_FCNS_3 | warning | no | A constructor has no output or more than one. |
//! | MP_FCNS_4 | warning | no | The file is a script. |
//! | MP_FCNS_5 | warning | no | A `%#ok<*ID>` directive suppresses a Code Analyzer message in the whole file. |
//! | MP_FCNS_6 | warning | no | A `get.<Property>` or `set.<Property>` method. |
//! | MP_FCNS_7 | warning | no | A mex function in the same folder lacks a build for a platform. |
//! | MP_FCNS_8 | warning | no | A block is nested deeper than five levels. |
//! | MP_FCNS_13 | warning | no | A loop variable is already the iterator of an earlier loop in the function. |
//! | MP_FCNS_16 | warning | no | A function is nested in another function. |
//! | MP_FCNS_18 | error | no | A method declared by signature has no implementation. |
//! | MP_FCNS_19 | error | no | The inputs or outputs of a declared method differ from its implementation. |
//! | MP_FCNS_21 | error | no | A loop variable is read after its loop. |
//! | MP_FCNS_22 | error | no | A function defined in the file is called with more inputs than it takes. |
//! | MP_FCNS_24 | warning | no | A call to a Java function, a Java package or an import of Java classes. |
//!
//! ## Parameters
//!
//! | Check | Parameter | Default | Meaning |
//! | ----- | --------- | ------- | ------- |
//! | MP_FCNS_1 | `main`, `local`, `method`, `nested` | see below | A table per kind of function with `max_complexity` and `max_lines`; 0 switches a limit off |
//! | MP_FCNS_2 | `max_inputs`, `max_outputs` | 5, 5 | Most inputs and outputs of a function |
//! | MP_FCNS_7 | `platforms` | `mexw32`, `mexw64`, `mexa64`, `mexmaci64\|mexmaca64` | Mex extensions that must exist for a mex function; `\|` separates alternatives |
//! | MP_FCNS_8 | `max_depth` | 5 | Deepest allowed nesting of blocks |
//! | MP_FCNS_24 | `functions`, `packages` | the Java functions and `java`, `javax`, `com`, `org` | What counts as Java |
//!
//! The defaults of MP_FCNS_1 are, as `max_complexity` and `max_lines`: main 20 and 100,
//! local 15 and 50, method 10 and 20, nested 5 and 10. Lines are code lines, without blank
//! and comment lines and without the lines of functions nested in the function.
//!
//! ```toml
//! [lint.rules.MP_FCNS_1.method]
//! max_lines = 40
//!
//! [lint.rules.MP_FCNS_2]
//! max_inputs = 6
//! ```
//!
//! ## What each check covers
//!
//! - MP_FCNS_1 counts one for the function, each `if`, `elseif`, `for`, `while`, `case`
//!   and `catch`, and each short-circuit operator.
//! - MP_FCNS_3 checks methods named like their class, also bare declarations.
//! - MP_FCNS_4 reports the first statement of a file that does not start with `function` or
//!   `classdef`. It cannot tell a project start-up script, the exception the standard allows.
//! - MP_FCNS_5 reads the rule as the file-wide `%#ok<*ID>` directive. Line directives and
//!   pragmas like `%#codegen` are left alone, and so is the reason in the comment.
//! - MP_FCNS_6 reports each `get.<Property>` and `set.<Property>` method.
//! - MP_FCNS_7 needs the folder: it reports calls to a function whose `.mex*` files sit
//!   next to the checked file. Mex files elsewhere on the path are not found.
//! - MP_FCNS_8 counts `if`, `for`, `parfor`, `while`, `switch` and `try`, not `case` or
//!   `catch`, and reports the statement that crosses the limit once.
//! - MP_FCNS_13 reports the second and later loop that uses a variable name within one
//!   function, nested loops included.
//! - MP_FCNS_16 reports every function inside another function, as the standard gives no count.
//! - MP_FCNS_18 and MP_FCNS_19 look at signatures in `methods` blocks that are not
//!   `Abstract`. The implementation is a function of that name in the same file, as the
//!   standard's examples show, or `name.m` in the `@Class` folder of the file. Both need the
//!   file on disk. The reverse direction (an implementation without a declaration) is not
//!   checked.
//! - MP_FCNS_21 reports reads of a loop variable after its loop with no assignment in
//!   between.
//! - MP_FCNS_22 covers the main, local and nested functions of the file and static methods
//!   called as `Class.method(...)`. Functions with `varargin` or name-value arguments are
//!   skipped, and so are calls with a `c{:}` argument. MATLAB's own Code Analyzer reports
//!   some of the same calls.
//! - MP_FCNS_24 reports calls of the Java functions, expressions that start with a Java
//!   package and imports of such packages. A variable with the name of a package is not Java.
//!
//! ## Examples
//!
//! ### Incorrect
//!
//! ```matlab
//! function total = sumAll(a, b, c, d, e, f)   % MP_FCNS_2: six inputs
//!     for i = 1:3
//!     end
//!     for i = 1:3                              % MP_FCNS_13: i is used again
//!     end
//!     total = i;                               % MP_FCNS_21: i read after the loop
//! end
//! ```
//!
//! ### Correct
//!
//! ```matlab
//! function total = sumAll(values)
//!     total = 0;
//!     for iFirst = 1:3
//!         total = total + values(iFirst);
//!     end
//!     for iSecond = 1:3
//!         total = total + values(iSecond);
//!     end
//! end
//! ```

use malt_core::{Config, Rule};

mod check_accessors;
mod check_arguments;
mod check_constructor;
mod check_declared;
mod check_extra_inputs;
mod check_iterator_outside;
mod check_iterator_reuse;
mod check_java;
mod check_mex;
mod check_nested;
mod check_nesting;
mod check_script;
mod check_signature_match;
mod check_size;
mod check_suppress;
mod common;
mod implementations;

use super::{Check, GroupEngine};

/// The checks of this group.
pub(crate) const CHECKS: &[Check] = &[
    Check {
        id: "MP_FCNS_1",
        level: 4,
        run: check_size::run,
    },
    Check {
        id: "MP_FCNS_2",
        level: 8,
        run: check_arguments::run,
    },
    Check {
        id: "MP_FCNS_3",
        level: 5,
        run: check_constructor::run,
    },
    Check {
        id: "MP_FCNS_4",
        level: 4,
        run: check_script::run,
    },
    Check {
        id: "MP_FCNS_5",
        level: 5,
        run: check_suppress::run,
    },
    Check {
        id: "MP_FCNS_6",
        level: 5,
        run: check_accessors::run,
    },
    Check {
        id: "MP_FCNS_7",
        level: 3,
        run: check_mex::run,
    },
    Check {
        id: "MP_FCNS_8",
        level: 5,
        run: check_nesting::run,
    },
    Check {
        id: "MP_FCNS_13",
        level: 5,
        run: check_iterator_reuse::run,
    },
    Check {
        id: "MP_FCNS_16",
        level: 5,
        run: check_nested::run,
    },
    Check {
        id: "MP_FCNS_18",
        level: 2,
        run: check_declared::run,
    },
    Check {
        id: "MP_FCNS_19",
        level: 2,
        run: check_signature_match::run,
    },
    Check {
        id: "MP_FCNS_21",
        level: 2,
        run: check_iterator_outside::run,
    },
    Check {
        id: "MP_FCNS_22",
        level: 1,
        run: check_extra_inputs::run,
    },
    Check {
        id: "MP_FCNS_24",
        level: 3,
        run: check_java::run,
    },
];

/// Factory constructor invoked by inventory.
pub fn from_config(config: &Config) -> Box<dyn Rule> {
    GroupEngine::boxed(
        "MP_FUNCTIONS_ENGINE",
        "MonkeyProof higher level rules",
        CHECKS,
        config,
    )
}

inventory::submit!(crate::RuleRegistration::new(
    "MP_FUNCTIONS_ENGINE",
    from_config
));
