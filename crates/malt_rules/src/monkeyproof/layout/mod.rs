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
//! note = "Not part of MATLAB's Code Analyzer. Off unless the monkeyproof preset or category is on."
//! ```
//!
//! ## Rule
//!
//! The checks of the layout and comments chapter of the
//! [MonkeyProof Coding Standard for MATLAB](https://doc.monkeyproofsolutions.nl/code-checker-for-matlab/monkeyproof-coding-standard-for-matlab/layout/layout.html) that can be checked automatically.
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
