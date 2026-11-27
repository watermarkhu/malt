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
