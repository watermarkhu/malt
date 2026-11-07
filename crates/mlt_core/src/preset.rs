//! Named configuration presets.
//!
//! A preset is a curated set of categories that are disabled by default.
//! Presets are selected via `[lint] preset = "..."` in a `.mlt.toml` file, the
//! `--preset` CLI flag, or the [`DEFAULT_PRESET`] when neither is specified.
//!
//! A preset layers on top of each rule's built-in default
//! ([`crate::Rule::enabled_by_default`]): a rule is on by default only if the
//! rule itself is default-on and the preset does not disable its category. The
//! `all` preset is the exception: it also turns on default-off engines.
//!
//! Presets are intentionally data-driven: adding a new one is a single row in
//! [`PRESETS`]. Explicit `[lint.categories]` / `[lint.rules]` entries always
//! take precedence over a preset, so a preset-disabled category can still be
//! re-enabled by the user.

use crate::rule::Category;

/// The preset used when neither `--preset` nor `[lint] preset` is given.
pub const DEFAULT_PRESET: &str = "mathworks";

/// A single named preset.
#[derive(Debug, Clone, Copy)]
pub struct PresetDef {
    /// Preset name, as used by `--preset` and `[lint] preset` (e.g. "all").
    pub name: &'static str,
    /// Categories disabled by this preset.
    pub disabled_categories: &'static [Category],
    /// Whether this preset also enables rules that are off by default
    /// (see [`crate::Rule::enabled_by_default`]).
    pub enables_default_off: bool,
}

impl PresetDef {
    /// Whether this preset disables `category`.
    pub fn disables(&self, category: Category) -> bool {
        self.disabled_categories.contains(&category)
    }

    /// Whether a rule in `category` is enabled when neither a per-rule nor a
    /// per-category setting applies. `rule_default` is the rule's
    /// [`crate::Rule::enabled_by_default`].
    pub fn default_enabled(&self, category: Category, rule_default: bool) -> bool {
        (rule_default || self.enables_default_off) && !self.disables(category)
    }
}

/// All built-in presets.
///
/// To add a preset, add a row here. Presets must be listed in
/// [`PRESETS`] to be resolvable by name.
pub static PRESETS: &[PresetDef] = &[
    PresetDef {
        name: "all",
        disabled_categories: &[],
        enables_default_off: true,
    },
    PresetDef {
        name: "mathworks",
        // Matches MATLAB Code Analyzer's factory configuration: the "Custom
        // Checks" complexity metrics and "Naming" conventions are opt-in.
        disabled_categories: &[Category::CustomChecks, Category::Naming],
        enables_default_off: false,
    },
    PresetDef {
        name: "recommended",
        // Errors and warnings on; Info-level style/suggestion checks off.
        disabled_categories: &[
            Category::Performance,
            Category::Readability,
            Category::Formatting,
            Category::SuggestedImprovements,
            Category::Naming,
            Category::CustomChecks,
        ],
        enables_default_off: false,
    },
];

/// Resolve a preset by name.
pub fn resolve(name: &str) -> Option<&'static PresetDef> {
    PRESETS.iter().find(|p| p.name == name)
}

/// Resolve the default preset.
pub fn default_preset() -> &'static PresetDef {
    resolve(DEFAULT_PRESET).expect("DEFAULT_PRESET must be in PRESETS")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_known_presets() {
        assert_eq!(resolve("all").unwrap().name, "all");
        assert_eq!(resolve("mathworks").unwrap().name, "mathworks");
        assert_eq!(resolve("recommended").unwrap().name, "recommended");
    }

    #[test]
    fn resolve_unknown_is_none() {
        assert!(resolve("bogus").is_none());
    }

    #[test]
    fn default_preset_is_mathworks() {
        assert_eq!(DEFAULT_PRESET, "mathworks");
        assert_eq!(default_preset().name, "mathworks");
    }

    #[test]
    fn all_disables_nothing() {
        let all = resolve("all").unwrap();
        assert!(!all.disables(Category::CustomChecks));
        assert!(!all.disables(Category::Naming));
        assert!(!all.disables(Category::Performance));
    }

    #[test]
    fn default_enabled_combines_rule_default_and_preset() {
        let mw = resolve("mathworks").unwrap();
        assert!(mw.default_enabled(Category::Bugs, true));
        assert!(!mw.default_enabled(Category::Naming, true));
        assert!(!mw.default_enabled(Category::CodeGeneration, false));

        let all = resolve("all").unwrap();
        assert!(all.default_enabled(Category::Naming, true));
        assert!(all.default_enabled(Category::CodeGeneration, false));
    }

    #[test]
    fn mathworks_disables_optin_groups() {
        let mw = resolve("mathworks").unwrap();
        assert!(mw.disables(Category::CustomChecks));
        assert!(mw.disables(Category::Naming));
        // Everything else stays enabled.
        assert!(!mw.disables(Category::Bugs));
        assert!(!mw.disables(Category::Readability));
        assert!(!mw.disables(Category::Formatting));
    }

    #[test]
    fn recommended_disables_style_noise() {
        let rec = resolve("recommended").unwrap();
        for c in [
            Category::Performance,
            Category::Readability,
            Category::Formatting,
            Category::SuggestedImprovements,
            Category::Naming,
            Category::CustomChecks,
        ] {
            assert!(rec.disables(c), "recommended should disable {c:?}");
        }
        // Real issues remain enabled.
        for c in [
            Category::SyntaxErrors,
            Category::LanguageSpecification,
            Category::Bugs,
            Category::GoodPractices,
            Category::UnsetVariables,
            Category::UnusedConstructions,
            Category::Compatibility,
        ] {
            assert!(!rec.disables(c), "recommended should keep {c:?}");
        }
    }
}
