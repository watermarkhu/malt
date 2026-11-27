//! # MP_STATEMENTS_ENGINE: MonkeyProof statements and expressions
//!
//! ```malt
//! id = "MP_STATEMENTS_ENGINE"
//! title = "MonkeyProof statements and expressions"
//! category = "monkeyproof"
//! severity = "info"
//! fix = false
//! slug = "monkeyproof-statements"
//! icon = "lucide/code"
//! note = "Not part of MATLAB's Code Analyzer. Off unless the monkeyproof preset or category is on."
//! ```
//!
//! ## Rule
//!
//! The checks of the statements and expressions chapter of the
//! [MonkeyProof Coding Standard for MATLAB](https://doc.monkeyproofsolutions.nl/code-checker-for-matlab/monkeyproof-coding-standard-for-matlab/statements/statements.html) that can be checked automatically.
//!
//! ## Check IDs
//!
//! | Check ID | Severity | Fix | Description |
//! | -------- | -------- | --- | ----------- |
//!
//! ## Examples
//!
//! ### Incorrect
//!
//! ```matlab
//! % See the checks above.
//! ```

use malt_core::{Config, Rule};

use super::{Check, GroupEngine};

/// The checks of this group.
pub(crate) const CHECKS: &[Check] = &[];

/// Factory constructor invoked by inventory.
pub fn from_config(config: &Config) -> Box<dyn Rule> {
    GroupEngine::boxed(
        "MP_STATEMENTS_ENGINE",
        "MonkeyProof statements and expressions",
        CHECKS,
        config,
    )
}

inventory::submit!(crate::RuleRegistration::new(
    "MP_STATEMENTS_ENGINE",
    from_config
));
